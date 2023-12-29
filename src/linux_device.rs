use serde::Deserialize;

use crate::win_device::{LinuxDataFormat, WinCsrk, WinEDiv, WinERand, WinIrk, WinLtk};

#[derive(Debug)]
pub(crate) struct LinuxDevice {
    pub info: LinuxInfo,
}

#[derive(Deserialize, Debug)]
pub(crate) struct LinuxInfo {
    #[serde(rename = "LinkKey")]
    pub link_key: Option<LinkKey>,
    #[serde(rename = "IdentityResolvingKey")]
    pub identity_resolving_key: Option<IdentityResolvingKey>,
    #[serde(rename = "SlaveLongTermKey")]
    pub slave_long_term_key: Option<SlaveLongTermKey>,
    #[serde(rename = "PeripheralLongTermKey")]
    pub peripheral_long_term_key: Option<PeripheralLongTermKey>,
    #[serde(rename = "LocalSignatureKey")]
    pub local_signature_key: Option<LocalSignatureKey>,
    #[serde(rename = "LongTermKey")]
    pub long_term_key: Option<LongTermKey>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct LinkKey {
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
    pub fn recreate(&self, ltk: &WinLtk) -> Self {
        Self {
            key: ltk.get_linux_format(),
            r#type: self.r#type.clone(),
            pin_length: self.pin_length.clone(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub(crate) struct IdentityResolvingKey {
    /// Key=786DC4332D385A48C4E718FE0B84FF20
    #[serde(rename = "Key")]
    key: String,
}

impl IdentityResolvingKey {
    pub fn recreate(&self, irk: &WinIrk) -> Self {
        Self {
            key: irk.get_linux_format(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub(crate) struct SlaveLongTermKey {
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

impl SlaveLongTermKey {
    pub fn recreate(&self, ltk: &WinLtk) -> Self {
        Self {
            key: ltk.get_linux_format(),
            authenticated: self.authenticated.clone(),
            enc_size: self.enc_size.clone(),
            e_div: self.e_div.clone(),
            rand: self.rand.clone(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub(crate) struct PeripheralLongTermKey {
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

impl PeripheralLongTermKey {
    pub fn recreate(&self, ltk: &WinLtk) -> Self {
        Self {
            key: ltk.get_linux_format(),
            authenticated: self.authenticated.clone(),
            enc_size: self.enc_size.clone(),
            e_div: self.e_div.clone(),
            rand: self.rand.clone(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub(crate) struct LocalSignatureKey {
    /// Key=128515400334819AA35B2D6C010BCEB1
    #[serde(rename = "Key")]
    key: String,
}

impl LocalSignatureKey {
    pub fn recreate(&self, csrk: &WinCsrk) -> Self {
        Self {
            key: csrk.get_linux_format(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub(crate) struct LongTermKey {
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

impl LongTermKey {
    pub fn recreate(&self, ltk: &WinLtk, e_rand: &WinERand, e_div: &WinEDiv) -> Self {
        Self {
            key: ltk.get_linux_format(),
            authenticated: self.authenticated.clone(),
            enc_size: self.authenticated.clone(),
            e_div: e_div.get_linux_format(),
            rand: e_rand.get_linux_format(),
        }
    }
}
