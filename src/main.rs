use log::{debug, warn};
use simple_logger::SimpleLogger;
use std::{
    collections::HashMap,
    fs::read_to_string,
    path::Path,
    process::{Command, Stdio},
};
use win_device::{LinuxDataFormat, WinDevice};

use crate::{
    linux_device::LinuxDevice,
    win_device::{WinInfo, WinMac, WinMeta},
};

mod linux_device;
mod win_device;

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
                meta: WinMeta {
                    mac: WinMac(addr),
                    adapter_mac: WinMac(adapter_addr),
                },
            }
        })
        .collect();

    devices
}

fn update_linux_devices(win_devices: Vec<WinDevice>) {
    win_devices.iter().for_each(|d| {
        println!("device {:?}", d);

        let dev_path = Path::new(LINUX_BT_DIR)
            .join(&d.meta.adapter_mac.get_linux_format())
            .join(&d.meta.mac.get_linux_format());
        if !dev_path.exists() {
            warn!(
                "device from windows with mac {} is not connected in linux",
                d.meta.mac.get_linux_format()
            );
            return;
        }

        let info_path = Path::new(&dev_path).join("info");
        let info_str = read_to_string(&info_path).expect("no info file in mac folder");
        println!("str {}", info_str);
        let mut dev = LinuxDevice {
            info: serde_ini::from_str(&info_str).expect("linux devices should be okay"),
        };

        if let Some(link_key) = dev.info.link_key.as_mut() {
            let _ = std::mem::replace(link_key, link_key.recreate(&d.info.ltk));
        }

        if let Some(identity_resolving_key) = dev.info.identity_resolving_key.as_mut() {
            if let Some(irk) = d.info.irk.as_ref() {
                let _ =
                    std::mem::replace(identity_resolving_key, identity_resolving_key.recreate(irk));
            }
        }

        if let Some(peripheral_long_term_key) = dev.info.peripheral_long_term_key.as_mut() {
            let _ = std::mem::replace(
                peripheral_long_term_key,
                peripheral_long_term_key.recreate(&d.info.ltk),
            );
        }

        if let Some(slave_long_term_key) = dev.info.slave_long_term_key.as_mut() {
            let _ = std::mem::replace(
                slave_long_term_key,
                slave_long_term_key.recreate(&d.info.ltk),
            );
        }

        if let Some(local_signature_key) = dev.info.local_signature_key.as_mut() {
            if let Some(csrk) = d.info.csrk.as_ref() {
                let _ = std::mem::replace(local_signature_key, local_signature_key.recreate(csrk));
            }
        }

        if let Some(long_term_key) = dev.info.long_term_key.as_mut() {
            let _ = std::mem::replace(
                long_term_key,
                long_term_key.recreate(&d.info.ltk, &d.info.e_rand, &d.info.e_div),
            );
        }
    });
}
