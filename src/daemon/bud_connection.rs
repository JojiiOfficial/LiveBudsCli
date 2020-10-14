use async_std::io::prelude::*;
use async_std::os::unix::net::UnixStream;
use bluetooth_serial_port_async::BtSocket;
use galaxy_buds_live_rs::message;
use galaxy_buds_live_rs::message::bud_property::{BudProperty, EqualizerType, Placement};

use serde::ser::{Serialize, SerializeStruct, Serializer};

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
    pub address: String,
    pub batt_left: i8,
    pub batt_right: i8,
    pub batt_case: i8,
    pub placement_left: Placement,
    pub placement_right: Placement,
    pub equalizer_type: EqualizerType,
    pub touchpads_blocked: bool,
    pub noise_reduction: bool,
    pub did_battery_notify: bool,
}

impl BudsInfo {
    pub fn new<S: AsRef<str>>(stream: UnixStream, address: S) -> Self {
        Self {
            stream,
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

impl std::fmt::Debug for BudsInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]: batt_left: {}, batt_right: {}, batt_case: {}, placement_left: {:?}, placement_right: {:?}, equalizer_type: {:?}, touchpads_blocked: {}, noise_reduction: {}",
               self.address,self.batt_left, self.batt_right, self.batt_case, self.placement_left, self.placement_right, self.equalizer_type, self.touchpads_blocked, self.noise_reduction)
    }
}

impl Serialize for BudsInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("BudsInfo", 8)?;
        state.serialize_field("batt_left", &self.batt_left)?;
        state.serialize_field("batt_right", &self.batt_right)?;
        state.serialize_field("batt_case", &self.batt_case)?;
        state.serialize_field("placement_left", &self.placement_left.encode())?;
        state.serialize_field("placement_right", &self.placement_right.encode())?;
        state.serialize_field("equalizer_type", &self.equalizer_type.encode())?;
        state.serialize_field("touchpads_blocked", &self.touchpads_blocked)?;
        state.serialize_field("noise_reduction", &self.noise_reduction)?;
        state.end()
    }
}
