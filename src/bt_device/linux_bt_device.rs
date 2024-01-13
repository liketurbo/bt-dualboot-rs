use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct LinuxInfo {
    #[serde(rename = "General")]
    pub general: Option<General>,
    #[serde(rename = "DeviceID")]
    pub device_id: Option<DeviceId>,
    #[serde(rename = "ConnectionParameters")]
    pub connection_parameters: Option<ConnectionParameters>,
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

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct General {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Appearance")]
    pub appearance: Option<String>,
    #[serde(rename = "AddressType")]
    pub address_type: String,
    #[serde(rename = "SupportedTechnologies")]
    pub supported_technologies: String,
    #[serde(rename = "Trusted")]
    pub trusted: String,
    #[serde(rename = "Blocked")]
    pub blocked: String,
    #[serde(rename = "WakeAllowed")]
    pub wake_allowed: Option<String>,
    #[serde(rename = "Services")]
    pub services: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DeviceId {
    #[serde(rename = "Source")]
    source: String,
    #[serde(rename = "Vendor")]
    vendor: String,
    #[serde(rename = "Product")]
    product: String,
    #[serde(rename = "Version")]
    version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ConnectionParameters {
    #[serde(rename = "MinInterval")]
    min_interval: String,
    #[serde(rename = "MaxInterval")]
    max_interval: String,
    #[serde(rename = "Latency")]
    latency: String,
    #[serde(rename = "Timeout")]
    timeout: String,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct IdentityResolvingKey {
    /// Key=786DC4332D385A48C4E718FE0B84FF20
    #[serde(rename = "Key")]
    key: String,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct LocalSignatureKey {
    /// Key=128515400334819AA35B2D6C010BCEB1
    #[serde(rename = "Key")]
    key: String,
}

#[derive(Serialize, Deserialize, Debug)]
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