use serde::Deserialize;

#[derive(Debug)]
pub(crate) struct WinDevice {
    pub info: WinInfo,
    pub meta: WinMeta,
}

#[derive(Debug)]
pub(crate) struct WinMeta {
    /// c8290a11f4c2
    pub mac: WinMac,
    /// c0fbf9601c13
    pub adapter_mac: WinMac,
}

#[derive(Debug)]
pub(crate) struct WinMac(pub(crate) String);

impl LinuxDataFormat for WinMac {
    fn get_linux_format(&self) -> String {
        let mut duo_holder = vec![];
        let mut new_addr = vec![];

        let new_format = self
            .0
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

/// Bluetooth device information from Windows Registry
#[derive(Deserialize, Debug)]
pub(crate) struct WinInfo {
    /// "AuthReq": "dword:0000002d"
    #[serde(rename = "AuthReq")]
    pub auth_req: String,
    /// "ERand": "hex(b):00,00,00,00,00,00,00,00"
    #[serde(rename = "ERand")]
    pub e_rand: WinERand,
    /// "LTK": "hex:c2,90,19,3b,1e,be,c7,d0,18,c6,4f,e9,67,ad,6b,d5"
    #[serde(rename = "LTK")]
    pub ltk: WinLtk,
    /// "KeyLength": "dword:00000000"
    #[serde(rename = "KeyLength")]
    pub key_length: String,
    /// "EDIV": "dword:00000000"
    #[serde(rename = "EDIV")]
    pub e_div: WinEDiv,
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
    pub irk: Option<WinIrk>,
    /// "CSRK": "hex:fc,ea,f8,3e,e3,ee,ee,d0,96,61,96,2a,6e,b0,33,8a"
    pub csrk: Option<WinCsrk>,
}

pub(crate) trait LinuxDataFormat {
    fn get_linux_format(&self) -> String;
}

#[derive(Deserialize, Debug)]
pub(crate) struct WinLtk(String);

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
pub(crate) struct WinERand(String);

impl LinuxDataFormat for WinERand {
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

#[derive(Deserialize, Debug)]
pub(crate) struct WinEDiv(String);

impl LinuxDataFormat for WinEDiv {
    fn get_linux_format(&self) -> String {
        self.0[6..].to_string()
    }
}

#[derive(Deserialize, Debug)]
pub(crate) struct WinIrk(String);

impl LinuxDataFormat for WinIrk {
    fn get_linux_format(&self) -> String {
        self.0[4..]
            .to_uppercase()
            .chars()
            .filter(|c| *c != ',')
            .collect()
    }
}

#[derive(Deserialize, Debug)]
pub(crate) struct WinCsrk(String);

impl LinuxDataFormat for WinCsrk {
    fn get_linux_format(&self) -> String {
        self.0[4..]
            .to_uppercase()
            .chars()
            .filter(|c| *c != ',')
            .collect()
    }
}