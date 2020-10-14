use async_std::io::prelude::*;
use async_std::os::unix::net::UnixStream;
use bluetooth_serial_port_async::BtSocket;
use galaxy_buds_live_rs::message;
use galaxy_buds_live_rs::message::bud_property::{BudProperty, EqualizerType, Placement};

use serde_derive::{Deserialize, Serialize};

/// An active connection to a pair of buds
#[derive(Debug)]
pub struct BudsConnection {
    pub addr: String,
    pub socket: BtSocket,
    pub fd: i32,
}

#[derive(Debug)]
pub struct ConnectionEventInfo {
    pub addr: String,
    pub connected: bool,
}

impl ConnectionEventInfo {
    pub fn new(addr: String, connected: bool) -> Self {
        ConnectionEventInfo { addr, connected }
    }
}

/// Informations about a connected pair
/// of Galaxy Buds live
pub struct BudsInfo {
    pub stream: UnixStream,
    pub inner: BudsInfoInner,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BudsInfoInner {
    pub address: String,
    pub batt_left: i8,
    pub batt_right: i8,
    pub batt_case: i8,
    #[serde(with = "placement_dser")]
    pub placement_left: Placement,
    #[serde(with = "placement_dser")]
    pub placement_right: Placement,
    #[serde(with = "equalizer_dser")]
    pub equalizer_type: EqualizerType,
    pub touchpads_blocked: bool,
    pub noise_reduction: bool,
    pub did_battery_notify: bool,
}

impl BudsInfo {
    pub fn new<S: AsRef<str>>(stream: UnixStream, address: S) -> Self {
        Self {
            stream,
            inner: BudsInfoInner {
                address: address.as_ref().to_owned(),
                batt_left: 0,
                batt_right: 0,
                batt_case: 0,
                placement_left: Placement::Undetected,
                placement_right: Placement::Undetected,
                equalizer_type: EqualizerType::Undetected,
                touchpads_blocked: false,
                noise_reduction: false,
                did_battery_notify: false,
            },
        }
    }

    // Send a message to the earbuds
    pub async fn send<T>(&self, msg: T) -> Result<(), String>
    where
        T: message::Payload,
    {
        let mut stream = &self.stream;
        if let Err(err) = stream.write(&msg.to_byte_array()).await {
            return Err(err.to_string());
        }

        Ok(())
    }
}

mod placement_dser {
    use galaxy_buds_live_rs::message::bud_property::{BudProperty, Placement};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(placement: &Placement, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        return s.serialize_u8(placement.encode());
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Placement, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: u8 = u8::deserialize(deserializer)?;
        return Ok(Placement::decode(s));
    }
}

mod equalizer_dser {
    use galaxy_buds_live_rs::message::bud_property::{BudProperty, EqualizerType};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(equalizer_type: &EqualizerType, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        return s.serialize_u8(equalizer_type.encode());
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<EqualizerType, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: u8 = u8::deserialize(deserializer)?;
        return Ok(EqualizerType::decode(s));
    }
}
