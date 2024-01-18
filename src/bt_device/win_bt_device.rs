use std::collections::HashMap;

use log::debug;
use serde::Deserialize;

use super::uni_bt_device;

pub struct BtDeviceBuilder {
    address: Option<KeyAddress>,
    parent_address: Option<KeyAddress>,
    ltk: Option<Ltk>,
    entries51: Option<BtDevice51>,
}

impl BtDeviceBuilder {
    pub fn new() -> Self {
        Self {
            address: None,
            parent_address: None,
            ltk: None,
            entries51: None,
        }
    }

    /// Accepts ltk in the format `"hex:c2,90,19,3b,1e,be,c7,d0,18,c6,4f,e9,67,ad,6b,d5"`
    pub fn ltk(mut self, ltk: String) -> Self {
        self.ltk = Some(Ltk(ltk));
        self
    }

    /// Accepts address in the format `"4c875d26dc9f"`
    pub fn address(mut self, address_b: String) -> Self {
        self.address = Some(KeyAddress(address_b));
        self
    }

    /// Accepts address in the format `"4c875d26dc9f"`
    pub fn parent_address(mut self, parent_address: String) -> Self {
        self.parent_address = Some(KeyAddress(parent_address));
        self
    }

    /// Accepts hashmap with Bluetooth 5.1 values
    ///
    /// ## Example
    /// ```
    /// "AuthReq": "dword:0000002d"
    /// "ERand": "hex(b):00,00,00,00,00,00,00,00"
    /// "LTK": "hex:c2,90,19,3b,1e,be,c7,d0,18,c6,4f,e9,67,ad,6b,d5"
    /// "KeyLength": "dword:00000000"
    /// "EDIV": "dword:00000000"
    /// "AddressType": "dword:00000001"
    /// "MasterIRKStatus": "dword:00000001"
    /// "IRK": "hex:fc,ea,f8,3e,e3,ee,ee,d0,96,61,96,2a,6e,b0,33,8a"
    /// "CSRK": "hex:fc,ea,f8,3e,e3,ee,ee,d0,96,61,96,2a,6e,b0,33,8a"
    /// ```
    pub fn entries51(mut self, entries51: HashMap<String, String>) -> Self {
        let s = serde_ini::to_string(&entries51).expect("already deserialized before so it's okay");
        let bt_device51: BtDevice51 = serde_ini::from_str(&s).expect("problem WinDevice struct");
        self.entries51 = Some(bt_device51);
        self
    }

    pub fn build(self) -> uni_bt_device::UniBtDevice {
        let parent_address: uni_bt_device::Address =
            if let Some(parent_address) = self.parent_address {
                parent_address.into()
            } else {
                panic!("address of bluetooth adapter is not provided");
            };

        let address: uni_bt_device::Address = if let Some(address) = self.address {
            address.into()
        } else {
            if let Some(entries51) = self.entries51.as_ref() {
                entries51.address.clone().into()
            } else {
                panic!("address of bluetooth device is not provided");
            }
        };

        let ltk: uni_bt_device::Ltk = if let Some(ltk) = self.ltk {
            ltk.into()
        } else {
            if let Some(entries51) = self.entries51.as_ref() {
                entries51.ltk.clone().into()
            } else {
                panic!("ltk of bluetooth device is not provided");
            }
        };

        let e_rand: Option<uni_bt_device::ERand> = if let Some(entries51) = self.entries51.as_ref() {
            Some(entries51.e_rand.clone().into())
        } else {
            None
        };

        let e_div: Option<uni_bt_device::EDiv> = if let Some(entries51) = self.entries51.as_ref() {
            Some(entries51.e_div.clone().into())
        } else {
            None
        };

        let irk = if let Some(entries51) = self.entries51.as_ref() {
            entries51.irk.clone().map(|v| v.into())
        } else {
            None
        };

        let csrk = if let Some(entries51) = self.entries51.as_ref() {
            entries51.csrk.clone().map(|v| v.into())
        } else {
            None
        };

        uni_bt_device::UniBtDevice {
            address,
            parent_address,
            ltk,
            e_rand,
            e_div,
            irk,
            csrk,
        }
    }
}

#[derive(Deserialize, Debug)]
struct BtDevice51 {
    /// "AuthReq": "dword:0000002d"
    #[serde(rename = "AuthReq")]
    pub auth_req: String,
    /// "ERand": "hex(b):00,00,00,00,00,00,00,00"
    #[serde(rename = "ERand")]
    pub e_rand: ERand,
    /// "LTK": "hex:c2,90,19,3b,1e,be,c7,d0,18,c6,4f,e9,67,ad,6b,d5"
    #[serde(rename = "LTK")]
    pub ltk: Ltk,
    /// "KeyLength": "dword:00000000"
    #[serde(rename = "KeyLength")]
    pub key_length: String,
    /// "EDIV": "dword:00000000"
    #[serde(rename = "EDIV")]
    pub e_div: EDiv,
    /// "AddressType": "dword:00000001"
    #[serde(rename = "AddressType")]
    pub address_type: Option<String>,
    /// "Address": "hex(b):c1,f4,11,0a,29,c8,00,00"
    #[serde(rename = "Address")]
    pub address: Address,
    /// "MasterIRKStatus": "dword:00000001"
    #[serde(rename = "MasterIRKStatus")]
    pub master_irk_status: Option<String>,
    /// "IRK": "hex:fc,ea,f8,3e,e3,ee,ee,d0,96,61,96,2a,6e,b0,33,8a"
    #[serde(rename = "IRK")]
    pub irk: Option<Irk>,
    /// "CSRK": "hex:fc,ea,f8,3e,e3,ee,ee,d0,96,61,96,2a,6e,b0,33,8a"
    pub csrk: Option<Csrk>,
}

#[derive(Deserialize, Debug, Clone)]
struct Ltk(String);

impl From<Ltk> for uni_bt_device::Ltk {
    /// "hex:c2,90,19,3b,1e,be,c7,d0,18,c6,4f,e9,67,ad,6b,d5" -> [u8; 16]
    fn from(value: Ltk) -> Self {
        let arr = win_reged_helpers::hex_to_bytes(&value.0);
        debug!("win ltk {:?} -> {:?}", value.0, arr);
        Self(arr)
    }
}

#[derive(Deserialize, Debug, Clone)]
struct ERand(String);

impl From<ERand> for uni_bt_device::ERand {
    /// "hex(b):00,00,00,00,00,00,00,00" -> [u8; 8]
    fn from(value: ERand) -> Self {
        let arr = win_reged_helpers::hex_b_to_bytes(&value.0);
        debug!("win e_rand {:?} -> {:?}", value.0, arr);
        Self(arr)
    }
}

#[derive(Deserialize, Debug, Clone)]
struct EDiv(String);

impl From<EDiv> for uni_bt_device::EDiv {
    /// "dword:00000000" -> [u8; 4]
    fn from(value: EDiv) -> Self {
        let arr = win_reged_helpers::dword_to_bytes(&value.0);
        debug!("win e_div {:?} -> {:?}", value.0, arr);
        Self(arr)
    }
}

#[derive(Deserialize, Debug, Clone)]
struct Address(String);

impl From<Address> for uni_bt_device::Address {
    /// "hex(b):c1,f4,11,0a,29,c8,00,00" -> [u8; 6]
    fn from(value: Address) -> Self {
        let arr: [u8; 6] = win_reged_helpers::hex_b_to_bytes(&value.0)
            .into_iter()
            .rev()
            .skip(2)
            .collect::<Vec<_>>()
            .try_into()
            .expect("invalid address length");
        debug!("win address {:?} -> {:?}", value.0, arr);
        Self(arr)
    }
}

#[derive(Deserialize, Debug, Clone)]
struct Irk(String);

impl From<Irk> for uni_bt_device::Irk {
    /// "hex:fc,ea,f8,3e,e3,ee,ee,d0,96,61,96,2a,6e,b0,33,8a" -> [u8; 16]
    fn from(value: Irk) -> Self {
        let arr = win_reged_helpers::hex_to_bytes(&value.0);
        debug!("win irk {:?} -> {:?}", value.0, arr);
        Self(arr)
    }
}

#[derive(Deserialize, Debug, Clone)]
struct Csrk(String);

impl From<Csrk> for uni_bt_device::Csrk {
    /// "hex:fc,ea,f8,3e,e3,ee,ee,d0,96,61,96,2a,6e,b0,33,8a" -> [u8; 16]
    fn from(value: Csrk) -> Self {
        let arr = win_reged_helpers::hex_to_bytes(&value.0);
        debug!("win csrk {:?} -> {:?}", value.0, arr);
        Self(arr)
    }
}

struct KeyAddress(String);

impl From<KeyAddress> for uni_bt_device::Address {
    /// "c0fbf9601c13" -> [u8; 6]
    fn from(value: KeyAddress) -> Self {
        let bytes: Vec<_> = value
            .0
            .as_bytes()
            .chunks_exact(2)
            .map(|b| {
                let str = std::str::from_utf8(b).expect("invalid utf-8 characters");
                let byte = u8::from_str_radix(str, 16).expect("invalid hex digit");
                byte
            })
            .collect();
        let arr: [u8; 6] = bytes.try_into().expect("invalid mac address length");
        debug!("win mac {:?} -> {:?}", value.0, arr);
        Self(arr)
    }
}

mod win_reged_helpers {
    /// "hex:fc,ea,f8,3e,e3,ee,ee,d0,96,61,96,2a,6e,b0,33,8a" -> [u8; 16]
    pub fn hex_to_bytes(hex: &str) -> [u8; 16] {
        let bytes: Vec<_> = hex[4..]
            .split(',')
            .map(|s| u8::from_str_radix(s, 16).expect("invalid hex digit"))
            .collect();
        let arr: [u8; 16] = bytes.as_slice().try_into().expect("invalid bytes length");
        arr
    }

    /// "hex(b):00,00,00,00,00,00,00,00" -> [u8; 8]
    pub fn hex_b_to_bytes(hex: &str) -> [u8; 8] {
        let bytes: Vec<_> = hex[7..]
            .split(',')
            .map(|s| u8::from_str_radix(s, 16).expect("invalid hex digit"))
            .collect();
        let arr: [u8; 8] = bytes.as_slice().try_into().expect("invalid bytes length");
        arr
    }

    pub fn dword_to_bytes(dword: &str) -> [u8; 4] {
        let num = u32::from_str_radix(&dword[6..], 10).expect("invalid decimal number");
        let arr: [u8; 4] = num.to_ne_bytes();
        arr
    }
}
