use std::time::SystemTime;

use async_std::io::prelude::*;
use async_std::os::unix::net::UnixStream;
use galaxy_buds_rs::{
    message::{self, debug},
    model::Feature,
};
use galaxy_buds_rs::{
    message::{
        bud_property::{EqualizerType, Placement, TouchpadOption},
        extended_status_updated::ExtTapLockStatus,
    },
    model::Model,
};
use serde::{Deserialize, Serialize};

/// Informations about a connected pair
/// of Galaxy Buds live
pub struct BudsInfo {
    pub stream: UnixStream,
    pub inner: BudsInfoInner,
    pub last_debug: SystemTime,
    pub left_tp_hold_count: u8,
    pub right_tp_hold_count: u8,
    pub last_tp_update: SystemTime,
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
    #[serde(with = "DefModel")]
    pub model: Model,
    pub ambient_sound_enabled: bool,
    pub ambient_sound_volume: u8,
    pub extra_high_ambient_volume: bool,
    pub tab_lock_status: ExtTapLockStatus,
}

impl BudsInfo {
    pub fn new<S: AsRef<str>>(stream: UnixStream, address: S, model: Model) -> Self {
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
                model,
                ambient_sound_enabled: false,
                ambient_sound_volume: 0,
                extra_high_ambient_volume: false,
                tab_lock_status: ExtTapLockStatus::default(),
            },
            last_debug: SystemTime::now(),
            left_tp_hold_count: 0,
            right_tp_hold_count: 0,
            last_tp_update: SystemTime::now(),
        }
    }

    // shortcut for self.inner.model.has_feature
    pub fn has_feature(&self, feature: Feature) -> bool {
        self.inner.has_feature(feature)
    }

    /// resets the last_tp_update value
    pub fn reset_last_tp_update(&mut self) {
        self.left_tp_hold_count = 0;
        self.right_tp_hold_count = 0;
    }

    /// Returns the max ambient volume level for the given device
    pub fn get_max_ambientsound_volume_level(&self) -> u8 {
        match self.inner.model {
            Model::BudsPro => {
                if self.has_feature(Feature::ExtraHighAmbientVolume) {
                    4
                } else {
                    3
                }
            }
            Model::BudsLive => 0,
            Model::BudsPlus => {
                if self.has_feature(Feature::ExtraHighAmbientVolume) {
                    4
                } else {
                    3
                }
            }
            Model::Buds => 3,
            Model::Buds2 => 3,
            Model::BudsPro2 => {
                if self.has_feature(Feature::ExtraHighAmbientVolume) {
                    4
                } else {
                    3
                }
            }
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

impl BudsInfoInner {
    // shortcut for self.inner.model.has_feature
    pub fn has_feature(&self, feature: Feature) -> bool {
        self.model.has_feature(feature)
    }
}

// Serialize/Deserialize Placement
mod placement_dser {
    use galaxy_buds_rs::message::bud_property::{BudProperty, Placement};
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
    use galaxy_buds_rs::message::bud_property::{BudProperty, EqualizerType};
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
    use galaxy_buds_rs::message::bud_property::{BudProperty, TouchpadOption};
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

#[derive(Serialize, Deserialize)]
#[serde(remote = "Model")]
enum DefModel {
    Buds,
    BudsPlus,
    BudsLive,
    BudsPro,
    Buds2,
    BudsPro2,
}
