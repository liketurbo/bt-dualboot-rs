use bt_device::{linux_bt_device, uni_bt_device::UniBtDevice};
use clap::Parser;
use cli::Cli;
use error::CustomError;
use inquire::Select;
use log::{debug, info, warn};
use std::{
    collections::HashMap,
    fs::{read_to_string, File},
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use crate::{bt_device::win_bt_device, utils::is_valid_64_hex};

mod bt_device;
mod cli;
mod error;
mod utils;

const WINDOWS10_REGISTRY_PATH: &str = "Windows/System32/config/SYSTEM";
const REG_KEY_BLUETOOTH_PAIRING_KEYS: &str = r"ControlSet001\Services\BTHPORT\Parameters\Keys";
const LINUX_BT_DIR: &str = "/var/lib/bluetooth";

pub type CustomResult<T> = Result<T, CustomError>;

fn main() {
    let cli = Cli::parse();

    if cli.verbose {
        simple_logger::init_with_level(log::Level::Debug).expect("init logger");
    } else {
        simple_logger::init_with_level(log::Level::Warn).expect("init logger");
    }

    let bt_devices = get_reged_bt_devices().unwrap();
    update_linux_devices(bt_devices);
}

fn get_reged_bt_devices() -> CustomResult<Vec<UniBtDevice>> {
    let win_mounts = get_windows_mounts()?;
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

    let output = get_chntpw_export(win_mount)?;
    let output_clean = output
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
            let parent_address = p_k
                .split('\\')
                .collect::<Vec<&str>>()
                .into_iter()
                .rev()
                .next()
                .expect("checked by path_len")
                .to_string();

            h.into_iter()
                .filter(|(k, _)| is_valid_64_hex(k))
                .map(|(k, v)| {
                    win_bt_device::BtDeviceBuilder::new()
                        .address(k)
                        .ltk(v.clone())
                        .parent_address(parent_address.clone())
                        .build()
                })
                .collect::<Vec<_>>()
        })
        .collect();
    debug!("found {} device(s)", bt_values_1.len());

    let bt_values_2: Vec<_> = raw_values
        .into_iter()
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
            let parent_address = k
                .split("\\")
                .collect::<Vec<&str>>()
                .into_iter()
                .rev()
                .skip(1)
                .next()
                .expect("should have adapter's mac")
                .to_string();

            win_bt_device::BtDeviceBuilder::new()
                .entries51(v)
                .parent_address(parent_address)
                .build()
        })
        .collect();
    debug!("found {} separate device(s)", bt_values_2.len());

    let mut all_devices = vec![];
    all_devices.extend(bt_values_1);
    all_devices.extend(bt_values_2);
    Ok(all_devices)
}

fn get_chntpw_export(win_mount: &str) -> CustomResult<String> {
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
    Ok(output_str)
}

fn get_windows_mounts() -> CustomResult<Vec<String>> {
    let mounts = read_to_string("/proc/mounts").map_err(|e| e.into())?;

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

    Ok(win_mounts)
}

fn update_linux_devices(win_devices: Vec<UniBtDevice>) {
    win_devices
        .into_iter()
        .map(|d| {
            let d_path = Path::new(LINUX_BT_DIR)
                .join(linux_bt_device::BtAddress::from(d.parent_address.clone()).0)
                .join(linux_bt_device::BtAddress::from(d.address.clone()).0);
            (d, d_path)
        })
        .filter(|(d, d_path)| {
            if !d_path.exists() {
                warn!(
                    "device from windows with mac {} is not connected in linux",
                    linux_bt_device::BtAddress::from(d.address.clone()).0
                );
                false
            } else {
                true
            }
        })
        .map(|(uni_dev, d_path)| {
            let info_path = Path::new(&d_path).join("info");
            let info_str = read_to_string(&info_path).expect("no info file in mac folder");

            let linux_dev: linux_bt_device::BtDevice =
                serde_ini::from_str(&info_str).expect("info always for bt device");

            let mut builder = linux_bt_device::BtDeviceBuilder::new()
                .device(linux_dev)
                .ltk(uni_dev.ltk);

            if let Some(irk) = uni_dev.irk {
                builder = builder.irk(irk);
            }

            if let Some(csrk) = uni_dev.csrk {
                builder = builder.csrk(csrk);
            }

            if let Some(e_div) = uni_dev.e_div {
                builder = builder.e_div(e_div);
            }

            if let Some(e_rand) = uni_dev.e_rand {
                builder = builder.e_rand(e_rand);
            }

            let updated_linux_dev = builder.build();

            (updated_linux_dev, d_path)
        })
        .for_each(|(d, d_path)| {
            let str = serde_ini::to_string(&d).unwrap();
            let mut file = File::create(d_path.join("info")).expect("can't open info file");
            file.write_all(str.as_bytes())
                .expect("writing of update failed");
            info!("updated {:?} device", d_path);
        });
}
