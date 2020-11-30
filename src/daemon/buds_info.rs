use std::time::SystemTime;

use async_std::io::prelude::*;
use async_std::os::unix::net::UnixStream;
use galaxy_buds_live_rs::message::bud_property::{EqualizerType, Placement, TouchpadOption};
use galaxy_buds_live_rs::message::{self, debug};
use serde_derive::{Deserialize, Serialize};

/// Informations about a connected pair
/// of Galaxy Buds live
pub struct BudsInfo {
    pub stream: UnixStream,
    pub inner: BudsInfoInner,
    pub last_debug: SystemTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DebugInfo {
    pub voltage_left: f32,
    pub voltage_right: f32,
    pub temperature_left: f32,
    pub temperature_right: f32,
    pub current_left: f64,
    pub current_right: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BudsInfoInner {
    pub address: String,
    pub ready: bool,
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
    #[serde(with = "touchpad_option_dser")]
    pub touchpad_option_left: TouchpadOption,
    #[serde(with = "touchpad_option_dser")]
    pub touchpad_option_right: TouchpadOption,
    pub paused_music_earlier: bool,
    pub debug: DebugInfo,
}

impl BudsInfo {
    pub fn new<S: AsRef<str>>(stream: UnixStream, address: S) -> Self {
        Self {
            stream,
            inner: BudsInfoInner {
                address: address.as_ref().to_owned(),
                ready: false,
                batt_left: 0,
                batt_right: 0,
                batt_case: 0,
                placement_left: Placement::Undetected,
                placement_right: Placement::Undetected,
                equalizer_type: EqualizerType::Undetected,
                touchpads_blocked: false,
                noise_reduction: false,
                did_battery_notify: false,
                touchpad_option_left: TouchpadOption::Undetected,
                touchpad_option_right: TouchpadOption::Undetected,
                paused_music_earlier: false,
                debug: DebugInfo::default(),
            },
            last_debug: SystemTime::now(),
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

    pub async fn request_debug_data(&mut self) -> Result<(), String> {
        self.last_debug = SystemTime::now();
        self.send(debug::new(debug::DebugVariant::GetAllData)).await
    }
}

// Serialize/Deserialize Placement
mod placement_dser {
    use galaxy_buds_live_rs::message::bud_property::{BudProperty, Placement};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(placement: &Placement, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_u8(placement.encode())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Placement, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Placement::decode(u8::deserialize(deserializer)?))
    }
}

// Serialize/Deserialize EqualizerType
mod equalizer_dser {
    use galaxy_buds_live_rs::message::bud_property::{BudProperty, EqualizerType};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(equalizer_type: &EqualizerType, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_u8(equalizer_type.encode())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<EqualizerType, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(EqualizerType::decode(u8::deserialize(deserializer)?))
    }
}

// Serialize/Deserialize TouchpadOption
mod touchpad_option_dser {
    use galaxy_buds_live_rs::message::bud_property::{BudProperty, TouchpadOption};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(touchpad_option: &TouchpadOption, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_u8(touchpad_option.encode())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<TouchpadOption, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(TouchpadOption::decode(u8::deserialize(deserializer)?))
    }
}
