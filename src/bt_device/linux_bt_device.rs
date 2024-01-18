use serde::{Deserialize, Serialize};

use super::uni_bt_device;

pub struct BtDeviceBuilder {
    device: Option<BtDevice>,
    ltk: Option<uni_bt_device::Ltk>,
    e_rand: Option<uni_bt_device::ERand>,
    e_div: Option<uni_bt_device::EDiv>,
    irk: Option<uni_bt_device::Irk>,
    csrk: Option<uni_bt_device::Csrk>,
}

impl BtDeviceBuilder {
    pub fn new() -> Self {
        Self {
            device: None,
            ltk: None,
            e_rand: None,
            e_div: None,
            irk: None,
            csrk: None,
        }
    }

    pub fn device(mut self, device: BtDevice) -> Self {
        self.device = Some(device);
        self
    }

    pub fn ltk(mut self, ltk: uni_bt_device::Ltk) -> Self {
        self.ltk = Some(ltk);
        self
    }

    pub fn e_rand(mut self, e_rand: uni_bt_device::ERand) -> Self {
        self.e_rand = Some(e_rand);
        self
    }

    pub fn e_div(mut self, e_div: uni_bt_device::EDiv) -> Self {
        self.e_div = Some(e_div);
        self
    }

    pub fn irk(mut self, irk: uni_bt_device::Irk) -> Self {
        self.irk = Some(irk);
        self
    }

    pub fn csrk(mut self, csrk: uni_bt_device::Csrk) -> Self {
        self.csrk = Some(csrk);
        self
    }

    pub fn build(mut self) -> BtDevice {
        let mut device = if let Some(dev) = self.device.take() {
            dev
        } else {
            panic!("didn't provide existing device to build upon");
        };

        let ltk = if let Some(lt) = self.ltk.as_ref() {
            lt
        } else {
            panic!("didn't provide a new ltk to replace with");
        };

        if let Some(link_key) = device.link_key.as_mut() {
            let new_link_key = LinkKey {
                key: linux_bt_helpers::bytes_to_linux_hex_key(&ltk.0),
                r#type: link_key.r#type.clone(),
                pin_length: link_key.pin_length.clone(),
            };
            let _ = std::mem::replace(link_key, new_link_key);
        }

        if let Some(identity_resolving_key) = device.identity_resolving_key.as_mut() {
            if let Some(irk) = self.irk {
                let new_identity_resolving_key = IdentityResolvingKey {
                    key: linux_bt_helpers::bytes_to_linux_hex_key(&irk.0),
                };
                let _ = std::mem::replace(identity_resolving_key, new_identity_resolving_key);
            }
        }

        if let Some(peripheral_long_term_key) = device.peripheral_long_term_key.as_mut() {
            let new_peripheral_long_term_key = PeripheralLongTermKey {
                key: linux_bt_helpers::bytes_to_linux_hex_key(&ltk.0),
                authenticated: peripheral_long_term_key.authenticated.clone(),
                enc_size: peripheral_long_term_key.enc_size.clone(),
                e_div: peripheral_long_term_key.e_div.clone(),
                rand: peripheral_long_term_key.rand.clone(),
            };
            let _ = std::mem::replace(peripheral_long_term_key, new_peripheral_long_term_key);
        }

        if let Some(slave_long_term_key) = device.slave_long_term_key.as_mut() {
            let new_slave_long_term_key = SlaveLongTermKey {
                key: linux_bt_helpers::bytes_to_linux_hex_key(&ltk.0),
                authenticated: slave_long_term_key.authenticated.clone(),
                enc_size: slave_long_term_key.enc_size.clone(),
                e_div: slave_long_term_key.e_div.clone(),
                rand: slave_long_term_key.rand.clone(),
            };
            let _ = std::mem::replace(slave_long_term_key, new_slave_long_term_key);
        }

        if let Some(local_signature_key) = device.local_signature_key.as_mut() {
            if let Some(csrk) = self.csrk {
                let new_local_signature_key = LocalSignatureKey {
                    key: linux_bt_helpers::bytes_to_linux_hex_key(&csrk.0),
                };
                let _ = std::mem::replace(local_signature_key, new_local_signature_key);
            }
        }

        if let Some(long_term_key) = device.long_term_key.as_mut() {
            if let Some(e_rand) = self.e_rand {
                if let Some(e_div) = self.e_div {
                    let new_long_term_key = LongTermKey {
                        key: linux_bt_helpers::bytes_to_linux_hex_key(&ltk.0),
                        authenticated: long_term_key.authenticated.clone(),
                        enc_size: long_term_key.enc_size.clone(),
                        e_div: linux_bt_helpers::bytes_to_linux_hex_key(&e_div.0),
                        rand: linux_bt_helpers::bytes_to_linux_hex_key(&e_rand.0),
                    };
                    let _ = std::mem::replace(long_term_key, new_long_term_key);
                }
            }
        }

        device
    }
}

pub struct BtAddress(pub String);

impl From<uni_bt_device::Address> for BtAddress {
    fn from(value: uni_bt_device::Address) -> Self {
        let hex = linux_bt_helpers::bytes_to_linux_hex_address(&value.0);
        Self(hex)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BtDevice {
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
pub struct General {
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
pub struct DeviceId {
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
pub struct ConnectionParameters {
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
pub struct LinkKey {
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
pub struct IdentityResolvingKey {
    /// Key=786DC4332D385A48C4E718FE0B84FF20
    #[serde(rename = "Key")]
    key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SlaveLongTermKey {
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
pub struct PeripheralLongTermKey {
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
pub struct LocalSignatureKey {
    /// Key=128515400334819AA35B2D6C010BCEB1
    #[serde(rename = "Key")]
    key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LongTermKey {
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

mod linux_bt_helpers {
    pub fn bytes_to_linux_hex_key(bytes: &[u8]) -> String {
        bytes
            .iter()
            .map(|b| format!("{:02x}", b).to_uppercase())
            .collect()
    }

    pub fn bytes_to_linux_hex_address(bytes: &[u8]) -> String {
        bytes
            .iter()
            .map(|b| format!("{:02x}", b).to_uppercase())
            .collect::<Vec<_>>()
            .join(":")
    }
}
