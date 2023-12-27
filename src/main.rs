use log::{debug, warn};
use serde::Deserialize;
use simple_logger::SimpleLogger;
use std::{
    collections::HashMap,
    fs::read_to_string,
    path::Path,
    process::{Command, Stdio},
};

const WINDOWS10_REGISTRY_PATH: &str = "Windows/System32/config/SYSTEM";
const REG_KEY_BLUETOOTH_PAIRING_KEYS: &str = r"ControlSet001\Services\BTHPORT\Parameters\Keys";
const LINUX_BT_DIR: &str = "/var/lib/bluetooth";

fn main() {
    SimpleLogger::new().init().expect("init logger");

    let win_devices = get_win_devices();
    update_linux_devices(win_devices);
}

fn get_win_devices() -> Vec<WinDevice> {
    let mounts = read_to_string("/proc/mounts").expect("failed to read /proc/mounts");

    let win_mounts: Vec<_> = mounts
        .split('\n')
        .filter(|l| l.starts_with("/dev/") && !l.starts_with("/dev/loop"))
        .map(|l| {
            let mnt_point = l
                .split(' ')
                .skip(1)
                .next()
                .expect(&format!("no mounting point in the dev entry {}", l));
            mnt_point
        })
        .filter(|mnt_p| {
            Path::new(mnt_p)
                .join(Path::new(WINDOWS10_REGISTRY_PATH))
                .exists()
        })
        .collect();

    if win_mounts.is_empty() {
        panic!("didn't find any mounts with windows")
    }

    if win_mounts.len() > 1 {
        // TODO: handle multiple windows mounts
        warn!(
            "reading only 1 windows mount at {}",
            win_mounts.get(0).expect("checked for emptiness")
        );
    }

    let win_mount = win_mounts.get(0).expect("checked for emptiness");
    let win_reg = Path::new(win_mount).join(Path::new(WINDOWS10_REGISTRY_PATH));

    debug!(
        "running chntpw on the {} registry",
        win_reg.to_str().expect("checked existence")
    );
    let chntpw = Command::new("reged")
        .args([
            "-E",
            "-x",
            win_reg.to_str().expect("checked existence"),
            r"HKEY_LOCAL_MACHINE\SYSTEM",
            REG_KEY_BLUETOOTH_PAIRING_KEYS,
            "/dev/stdout",
        ])
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed reged execution");

    let output = chntpw
        .wait_with_output()
        .expect("failed to wait on reged output")
        .stdout;
    let output_str = String::from_utf8(output).expect("failed to stringify");
    let output_clean = output_str
        .split("\r\n")
        .skip(1)
        .take_while(|l| !l.starts_with("reged version"))
        .collect::<Vec<_>>()
        .join("\n");
    let raw_values: HashMap<String, HashMap<String, String>> =
        serde_ini::from_str(&output_clean).expect("deserializing ini file");

    let bt_values: HashMap<_, _> = raw_values
        .into_iter()
        .filter(|(k, _)| {
            // Ignore an empty set and bt adapters
            // HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Services\\BTHPORT\\Parameters\\Keys
            // HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Services\\BTHPORT\\Parameters\\Keys\\c0fbf9601c13
            let path_len = k.split("\\").count();

            if path_len < 9 {
                false
            } else {
                true
            }
        })
        .map(|(k, v)| {
            // Remove "" quotes around the keys
            // "AuthReq" -> AuthReq
            let n_h: HashMap<_, _> = v
                .into_iter()
                .map(|(n_k, n_v)| (n_k.trim_matches('"').to_string(), n_v))
                .collect();
            (k, n_h)
        })
        .map(|(k, v)| {
            let s = serde_ini::to_string(&v).expect("already deserialized before so it's okay");
            let win_device: WinInfo = serde_ini::from_str(&s).expect("problem WinDevice struct");
            (k, win_device)
        })
        .collect();
    debug!("found {} device(s)", bt_values.len());

    let devices: Vec<_> = bt_values
        .into_iter()
        .map(|(k, v)| {
            let mut iter = k.split("\\").collect::<Vec<&str>>().into_iter().rev();
            let addr = iter.next().expect("should have mac").to_string();
            let adapter_addr = iter.next().expect("should have adapter's mac").to_string();

            WinDevice {
                info: v,
                meta: WinMeta { addr, adapter_addr },
            }
        })
        .collect();

    devices
}

#[derive(Debug)]
struct WinDevice {
    pub info: WinInfo,
    pub meta: WinMeta,
}

impl WinDevice {
    pub fn get_linux_mac(&self) -> String {
        let mut duo_holder = vec![];
        let mut new_addr = vec![];

        let new_format = self
            .meta
            .addr
            .to_uppercase()
            .chars()
            .fold((&mut duo_holder, &mut new_addr), |acc, v| {
                acc.0.push(v);
                if !acc.0.is_empty() && acc.0.len() % 2 == 0 {
                    let c_2 = acc.0.pop().expect("checked with if");
                    let c_1 = acc.0.pop().expect("checked with if");
                    let comb = format!("{}{}", c_1, c_2);
                    acc.1.push(comb);
                }
                acc
            })
            .1
            .join(":");

        new_format
    }

    pub fn get_linux_adapter_mac(&self) -> String {
        let mut duo_holder = vec![];
        let mut new_addr = vec![];

        let new_format = self
            .meta
            .adapter_addr
            .to_uppercase()
            .chars()
            .fold((&mut duo_holder, &mut new_addr), |acc, v| {
                acc.0.push(v);
                if !acc.0.is_empty() && acc.0.len() % 2 == 0 {
                    let c_2 = acc.0.pop().expect("checked with if");
                    let c_1 = acc.0.pop().expect("checked with if");
                    let comb = format!("{}{}", c_1, c_2);
                    acc.1.push(comb);
                }
                acc
            })
            .1
            .join(":");

        new_format
    }
}

#[derive(Debug)]
struct WinMeta {
    /// c8290a11f4c2
    pub addr: String,
    /// c0fbf9601c13
    pub adapter_addr: String,
}

/// Bluetooth device information from Windows Registry
#[derive(Deserialize, Debug)]
struct WinInfo {
    /// "AuthReq": "dword:0000002d"
    #[serde(rename = "AuthReq")]
    pub auth_req: String,
    /// "ERand": "hex(b):00,00,00,00,00,00,00,00"
    #[serde(rename = "ERand")]
    pub e_rand: ERand,
    /// "LTK": "hex:c2,90,19,3b,1e,be,c7,d0,18,c6,4f,e9,67,ad,6b,d5"
    #[serde(rename = "LTK")]
    pub ltk: WinLtk,
    /// "KeyLength": "dword:00000000"
    #[serde(rename = "KeyLength")]
    pub key_length: String,
    /// "EDIV": "dword:00000000"
    #[serde(rename = "EDIV")]
    pub ediv: String,
    /// "AddressType": "dword:00000001"
    #[serde(rename = "AddressType")]
    pub address_type: Option<String>,
    /// "Address": "hex(b):c1,f4,11,0a,29,c8,00,00"
    #[serde(rename = "Address")]
    pub address: Option<String>,
    /// "MasterIRKStatus": "dword:00000001"
    #[serde(rename = "MasterIRKStatus")]
    pub master_irk_status: Option<String>,
    /// "IRK": "hex:fc,ea,f8,3e,e3,ee,ee,d0,96,61,96,2a,6e,b0,33,8a"
    #[serde(rename = "IRK")]
    pub irk: Option<String>,
}

trait LinuxDataFormat {
    fn get_linux_format(&self) -> String;
}

#[derive(Deserialize, Debug)]
struct WinLtk(String);

impl LinuxDataFormat for WinLtk {
    fn get_linux_format(&self) -> String {
        self.0[4..]
            .to_uppercase()
            .chars()
            .filter(|c| *c != ',')
            .collect()
    }
}

#[derive(Deserialize, Debug)]
struct ERand(String);

impl LinuxDataFormat for ERand {
    fn get_linux_format(&self) -> String {
        u64::from_str_radix(
            &self.0[7..]
                .to_string()
                .chars()
                .filter(|c| *c != ',')
                .collect::<String>(),
            16,
        )
        .expect("probably 64 bit number")
        .to_string()
    }
}

// IdentityResolvingKey, SlaveLongTermKey, PeripheralLongTermKey <- IRK, LTK, ERand, EDIV
// IdentityResolvingKey, LocalSignatureKey, LongTermKey, LongTermKey, LongTermKey, LongTermKey

fn update_linux_devices(win_devices: Vec<WinDevice>) {
    win_devices.iter().for_each(|d| {
        println!("device {:?}", d);
        println!("ltk(linux) {}", d.info.ltk.get_linux_format());

        let dev_path = Path::new(LINUX_BT_DIR)
            .join(&d.get_linux_adapter_mac())
            .join(&d.get_linux_mac());
        if !dev_path.exists() {
            warn!(
                "device from windows with mac {} is not connected in linux",
                d.get_linux_mac()
            );
            return;
        }

        let info_path = Path::new(&dev_path).join("info");
        let info_str = read_to_string(&info_path).expect("no info file in mac folder");
        println!("str {}", info_str);
        let dev = LinuxDevice {
            info: serde_ini::from_str(&info_str).expect("linux devices should be okay"),
        };
        println!("dev {:?}", dev);
    });
}

#[derive(Debug)]
struct LinuxDevice {
    info: LinuxInfo,
}

#[derive(Deserialize, Debug)]
struct LinuxInfo {
    #[serde(rename = "LinkKey")]
    link_key: Option<LinkKey>,
    #[serde(rename = "IdentityResolvingKey")]
    identity_resolving_key: Option<IdentityResolvingKey>,
    #[serde(rename = "SlaveLongTermKey")]
    slave_long_term_key: Option<SlaveLongTermKey>,
    #[serde(rename = "PeripheralLongTermKey")]
    peripheral_long_term_key: Option<PeripheralLongTermKey>,
    #[serde(rename = "LocalSignatureKey")]
    local_signature_key: Option<LocalSignatureKey>,
    #[serde(rename = "LongTermKey")]
    long_term_key: Option<LongTermKey>,
}

#[derive(Deserialize, Debug)]
struct LinkKey {
    /// Key=786DC4332D385A48C4E718FE0B84FF20
    #[serde(rename = "Key")]
    key: String,
    /// Type=4
    #[serde(rename = "Type")]
    r#type: String,
    /// PINLength=0
    #[serde(rename = "PINLength")]
    pin_length: String,
}

impl LinkKey {
    fn from(link_key: &LinkKey, win_ltk: String) -> LinkKey {
        todo!()
    }
}

#[derive(Deserialize, Debug)]
struct IdentityResolvingKey {
    /// Key=786DC4332D385A48C4E718FE0B84FF20
    #[serde(rename = "Key")]
    key: String,
}

impl IdentityResolvingKey {
    fn from(
        identity_resolving_key: &IdentityResolvingKey,
        win_ltk: String,
    ) -> IdentityResolvingKey {
        todo!()
    }
}

#[derive(Deserialize, Debug)]
struct SlaveLongTermKey {
    /// Key=128515400334819AA35B2D6C010BCEB1
    #[serde(rename = "Key")]
    key: String,
    /// Authenticated=2
    #[serde(rename = "Authenticated")]
    authenticated: String,
    /// EncSize=16
    #[serde(rename = "EncSize")]
    enc_size: String,
    /// EDiv=0
    #[serde(rename = "EDiv")]
    e_div: String,
    /// Rand=0
    #[serde(rename = "Rand")]
    rand: String,
}

#[derive(Deserialize, Debug)]
struct PeripheralLongTermKey {
    /// Key=128515400334819AA35B2D6C010BCEB1
    #[serde(rename = "Key")]
    key: String,
    /// Authenticated=2
    #[serde(rename = "Authenticated")]
    authenticated: String,
    /// EncSize=16
    #[serde(rename = "EncSize")]
    enc_size: String,
    /// EDiv=0
    #[serde(rename = "EDiv")]
    e_div: String,
    /// Rand=0
    #[serde(rename = "Rand")]
    rand: String,
}

#[derive(Deserialize, Debug)]
struct LocalSignatureKey {
    /// Key=128515400334819AA35B2D6C010BCEB1
    #[serde(rename = "Key")]
    key: String,
}

#[derive(Deserialize, Debug)]
struct LongTermKey {
    /// Key=128515400334819AA35B2D6C010BCEB1
    #[serde(rename = "Key")]
    key: String,
    /// Authenticated=2
    #[serde(rename = "Authenticated")]
    authenticated: String,
    /// EncSize=16
    #[serde(rename = "EncSize")]
    enc_size: String,
    /// EDiv=0
    #[serde(rename = "EDiv")]
    e_div: String,
    /// Rand=0
    #[serde(rename = "Rand")]
    rand: String,
}

/// From hex:c2,90,19,3b,1e,be,c7,d0,18,c6,4f,e9,67,ad,6b,d5
/// To C290193B1EBEC7D018C64FE967AD6BD5
fn win_key_format_to_linux(win_key: String) -> String {
    todo!()
}
