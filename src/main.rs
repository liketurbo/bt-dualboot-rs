use log::{debug, info, warn};
use simple_logger::SimpleLogger;
use std::{
    collections::HashMap,
    fs::{read_to_string, File},
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use win_device::{LinuxDataFormat, WinDevice};

use crate::linux_device::LinuxDevice;

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
            let win_device: win_device::WinInfo =
                serde_ini::from_str(&s).expect("problem WinDevice struct");
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
                meta: win_device::WinMeta {
                    mac: win_device::WinMac(addr),
                    adapter_mac: win_device::WinMac(adapter_addr),
                },
            }
        })
        .collect();

    devices
}

fn update_linux_devices(win_devices: Vec<WinDevice>) {
    win_devices
        .iter()
        .map(|d| {
            let d_path = Path::new(LINUX_BT_DIR)
                .join(&d.meta.adapter_mac.get_linux_format())
                .join(&d.meta.mac.get_linux_format());
            (d, d_path)
        })
        .filter(|(d, d_path)| {
            if !d_path.exists() {
                warn!(
                    "device from windows with mac {} is not connected in linux",
                    d.meta.mac.get_linux_format()
                );
                false
            } else {
                true
            }
        })
        .map(|(win_dev, d_path)| {
            let info_path = Path::new(&d_path).join("info");
            let info_str = read_to_string(&info_path).expect("no info file in mac folder");

            let mut linux_dev = LinuxDevice {
                info: serde_ini::from_str(&info_str).expect("info always for bt device"),
            };

            if let Some(link_key) = linux_dev.info.link_key.as_mut() {
                let _ = std::mem::replace(link_key, link_key.recreate(&win_dev.info.ltk));
            }

            if let Some(identity_resolving_key) = linux_dev.info.identity_resolving_key.as_mut() {
                if let Some(irk) = win_dev.info.irk.as_ref() {
                    let _ = std::mem::replace(
                        identity_resolving_key,
                        identity_resolving_key.recreate(irk),
                    );
                }
            }

            if let Some(peripheral_long_term_key) = linux_dev.info.peripheral_long_term_key.as_mut()
            {
                let _ = std::mem::replace(
                    peripheral_long_term_key,
                    peripheral_long_term_key.recreate(&win_dev.info.ltk),
                );
            }

            if let Some(slave_long_term_key) = linux_dev.info.slave_long_term_key.as_mut() {
                let _ = std::mem::replace(
                    slave_long_term_key,
                    slave_long_term_key.recreate(&win_dev.info.ltk),
                );
            }

            if let Some(local_signature_key) = linux_dev.info.local_signature_key.as_mut() {
                if let Some(csrk) = win_dev.info.csrk.as_ref() {
                    let _ =
                        std::mem::replace(local_signature_key, local_signature_key.recreate(csrk));
                }
            }

            if let Some(long_term_key) = linux_dev.info.long_term_key.as_mut() {
                let _ = std::mem::replace(
                    long_term_key,
                    long_term_key.recreate(
                        &win_dev.info.ltk,
                        &win_dev.info.e_rand,
                        &win_dev.info.e_div,
                    ),
                );
            }

            (linux_dev, d_path)
        })
        .for_each(|(d, d_path)| {
            debug!("skipped update of device {:?}", d_path);
            /*
                       let str = serde_ini::to_string(&d.info).unwrap();
                       let mut file = File::create(d_path.join("info")).expect("can't open info file");
                       file.write_all(str.as_bytes()).expect("writing of update failed");
                       info!("updated {:?} device", d_path);
            */
        });
}
