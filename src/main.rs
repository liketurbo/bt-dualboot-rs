use clap::Parser;
use cli::Cli;
use inquire::{InquireError, Select};
use log::{debug, error, info, warn};
use simple_logger::SimpleLogger;
use std::{
    collections::HashMap,
    fs::{read_to_string, File},
    io::Write,
    path::Path,
    process::{Command, Stdio},
    string::FromUtf8Error,
};
use win_device::{LinuxDataFormat, WinDevice};

use crate::{linux_device::LinuxDevice, utils::is_valid_64_hex};

mod cli;
mod linux_device;
mod utils;
mod win_device;

const WINDOWS10_REGISTRY_PATH: &str = "Windows/System32/config/SYSTEM";
const REG_KEY_BLUETOOTH_PAIRING_KEYS: &str = r"ControlSet001\Services\BTHPORT\Parameters\Keys";
const LINUX_BT_DIR: &str = "/var/lib/bluetooth";

#[derive(Debug)]
pub enum CustomError {
    BtDualBootError(Box<dyn std::error::Error>),
    InquireError(InquireError),
    SerdeError(serde_ini::de::Error),
}

pub type CustomResult<T> = Result<T, CustomError>;

impl Into<CustomError> for InquireError {
    fn into(self) -> CustomError {
        CustomError::InquireError(self)
    }
}

impl Into<CustomError> for &str {
    fn into(self) -> CustomError {
        CustomError::BtDualBootError(self.into())
    }
}

impl Into<CustomError> for String {
    fn into(self) -> CustomError {
        CustomError::BtDualBootError(self.into())
    }
}

impl Into<CustomError> for std::io::Error {
    fn into(self) -> CustomError {
        CustomError::BtDualBootError(self.into())
    }
}

impl Into<CustomError> for FromUtf8Error {
    fn into(self) -> CustomError {
        CustomError::BtDualBootError(self.into())
    }
}

impl Into<CustomError> for serde_ini::de::Error {
    fn into(self) -> CustomError {
        CustomError::SerdeError(self)
    }
}

fn main() {
    let cli = Cli::parse();

    if cli.verbose {
        simple_logger::init_with_level(log::Level::Debug).expect("init logger");
    } else {
        simple_logger::init_with_level(log::Level::Warn).expect("init logger");
    }

    let win_devices = get_win_devices().unwrap();
    println!("{:#?}", win_devices);
    // update_linux_devices(win_devices);
}

fn get_win_devices() -> CustomResult<Vec<WinDevice>> {
    let win_mounts = get_windows_mounts();
    let win_mounts_str: Vec<_> = win_mounts.iter().map(|x| x.as_str()).collect();

    if win_mounts.is_empty() {
        return Err("no windows partitions".into());
    }
    debug!("found {} windows partition(s)", win_mounts.len());

    let win_mount = if win_mounts.len() == 1 {
        win_mounts.get(0).expect("checked len by 1")
    } else {
        Select::new(
            "multiple windows partitions detected. which one to use?",
            win_mounts_str,
        )
        .prompt()
        .map_err(|e| e.into())?
    };

    let win_reg = Path::new(win_mount).join(Path::new(WINDOWS10_REGISTRY_PATH));
    if !win_reg.exists() {
        return Err(format!("didn't find registry in windows {} partition", win_mount).into());
    }

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
        .map_err(|e| e.into())?;

    let output = chntpw.wait_with_output().map_err(|e| e.into())?.stdout;
    let output_str = String::from_utf8(output).map_err(|e| e.into())?;
    let output_clean = output_str
        .split("\r\n")
        .skip(1)
        .take_while(|l| !l.starts_with("reged version"))
        .collect::<Vec<_>>()
        .join("\n");
    let raw_values: HashMap<String, HashMap<String, String>> =
        serde_ini::from_str(&output_clean).map_err(|e| e.into())?;

    let bt_values_1: Vec<_> = raw_values
        .iter()
        .filter(|(k, _)| {
            // Match bt adapters
            // HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Services\\BTHPORT\\Parameters\\Keys\\c0fbf9601c13
            let path_len = k.split("\\").count();
            path_len == 8
        })
        .map(|(p_k, h)| {
            // Remove "" quotes around the keys
            // "AuthReq" -> AuthReq
            let n_h: HashMap<_, _> = h
                .into_iter()
                .map(|(k, v)| (k.trim_matches('"').to_string(), v))
                .collect();
            (p_k, n_h)
        })
        .flat_map(|(p_k, h)| {
            let adapter_mac = p_k
                .split('\\')
                .collect::<Vec<&str>>()
                .into_iter()
                .rev()
                .next()
                .expect("checked by path_len")
                .to_string();

            h.into_iter()
                .filter(|(k, v)| is_valid_64_hex(k))
                .map(|(k, v)| {
                    // 4c875d26dc9f -> hex(b):9f,dc,26,5d,87,4c,00,00
                    let addr = format!(
                        "hex(b):{},00,00",
                        k.as_bytes()
                            .chunks(2)
                            .map(|c| {
                                format!(
                                    "{:02x}",
                                    u8::from_str_radix(
                                        std::str::from_utf8(c).expect("derived before"),
                                        16
                                    )
                                    .expect("derived before")
                                )
                            })
                            .rev()
                            .collect::<Vec<_>>()
                            .join(",")
                    );

                    let mut raw_bt = HashMap::new();
                    raw_bt.insert("Address", addr);
                    raw_bt.insert("LTK", v.clone());
                    let s = serde_ini::to_string(&raw_bt)
                        .expect("already deserialized before so it's okay");
                    let info: win_device::WinInfo =
                        serde_ini::from_str(&s).expect("problem WinDevice struct");

                    WinDevice {
                        info,
                        meta: win_device::WinMeta {
                            adapter_mac: win_device::WinMac(adapter_mac.clone()),
                        },
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect();
    debug!("found {} device(s)", bt_values_1.len());

    let bt_values_2: Vec<_> = raw_values
        .iter()
        .filter(|(k, _)| {
            // Ignore an empty set and bt adapters
            // HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Services\\BTHPORT\\Parameters\\Keys
            // HKEY_LOCAL_MACHINE\\SYSTEM\\ControlSet001\\Services\\BTHPORT\\Parameters\\Keys\\c0fbf9601c13
            let path_len = k.split("\\").count();
            path_len > 8
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
        .into_iter()
        .map(|(k, v)| {
            let iter = k.split("\\").collect::<Vec<&str>>().into_iter().rev();
            let adapter_addr = iter
                .skip(1)
                .next()
                .expect("should have adapter's mac")
                .to_string();

            WinDevice {
                info: v,
                meta: win_device::WinMeta {
                    adapter_mac: win_device::WinMac(adapter_addr),
                },
            }
        })
        .collect();
    debug!("found {} separate device(s)", bt_values_2.len());

    let mut all_devices = vec![];
    all_devices.extend(bt_values_1);
    all_devices.extend(bt_values_2);
    Ok(all_devices)
}

fn get_windows_mounts() -> Vec<String> {
    let mounts = read_to_string("/proc/mounts").expect("failed to read /proc/mounts");

    let win_mounts: Vec<_> = mounts
        .split('\n')
        .filter(|l| l.starts_with("/dev/") && !l.starts_with("/dev/loop"))
        .map(|l| {
            let mnt_point = l
                .split(' ')
                .skip(1)
                .next()
                .expect(&format!("no mounting point in the dev entry {}", l))
                .to_string();
            mnt_point
        })
        .filter(|mnt_p| {
            Path::new(mnt_p)
                .join(Path::new(WINDOWS10_REGISTRY_PATH))
                .exists()
        })
        .collect();

    win_mounts
}

fn update_linux_devices(win_devices: Vec<WinDevice>) {
    win_devices
        .iter()
        .map(|d| {
            let d_path = Path::new(LINUX_BT_DIR)
                .join(&d.meta.adapter_mac.get_linux_format())
                .join(&d.info.address.get_linux_format());
            (d, d_path)
        })
        .filter(|(d, d_path)| {
            if !d_path.exists() {
                warn!(
                    "device from windows with mac {} is not connected in linux",
                    d.info.address.get_linux_format()
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
                if let Some(e_rand) = win_dev.info.e_rand.as_ref() {
                    if let Some(e_div) = win_dev.info.e_div.as_ref() {
                        let _ = std::mem::replace(
                            long_term_key,
                            long_term_key.recreate(&win_dev.info.ltk, e_rand, e_div),
                        );
                    }
                }
            }

            (linux_dev, d_path)
        })
        .for_each(|(d, d_path)| {
            let str = serde_ini::to_string(&d.info).unwrap();
            let mut file = File::create(d_path.join("info")).expect("can't open info file");
            file.write_all(str.as_bytes())
                .expect("writing of update failed");
            info!("updated {:?} device", d_path);
        });
}
