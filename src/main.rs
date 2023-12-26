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

fn main() {
    SimpleLogger::new().init().expect("init logger");

    let win_devices = get_win_devices();
    update_linux_devices(win_devices);
}

fn get_win_devices() -> Vec<Device> {
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
            let win_device: WinMetadata =
                serde_ini::from_str(&s).expect("problem WinDevice struct");
            (k, win_device)
        })
        .collect();
    debug!("found {} device(s)", bt_values.len());

    let devices: Vec<_> = bt_values
        .into_iter()
        .map(|(k, v)| {
            let mut iter = k.split("\\").collect::<Vec<&str>>().into_iter().rev();

            let mut duo_holder = vec![];
            let mut new_addr = vec![];

            let addr = iter
                .next()
                .expect("there is always key")
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

            duo_holder.clear();
            new_addr.clear();

            let adapter_addr = iter
                .next()
                .expect("there is always adapter's key")
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

            Device {
                addr,
                adapter_addr,
                metadata: v,
            }
        })
        .collect();

    devices
}

struct Device {
    /// C8:29:0A:11:F4:C2
    pub addr: String,
    /// C0:FB:F9:60:1C:13
    pub adapter_addr: String,
    /// Metadata from Windows Registry
    pub metadata: WinMetadata,
}

#[derive(Deserialize, Debug)]
struct WinMetadata {
    /// "AuthReq": "dword:0000002d"
    #[serde(rename = "AuthReq")]
    pub auth_req: String,
    /// "ERand": "hex(b):00,00,00,00,00,00,00,00"
    #[serde(rename = "ERand")]
    pub e_rand: String,
    /// "LTK": "hex:c2,90,19,3b,1e,be,c7,d0,18,c6,4f,e9,67,ad,6b,d5"
    #[serde(rename = "LTK")]
    pub ltk: String,
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

fn update_linux_devices(win_devices: Vec<Device>) {
    todo!()
}
