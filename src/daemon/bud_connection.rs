use async_std::os::unix::net::UnixStream;
use bluetooth_serial_port_async::BtSocket;
use galaxy_buds_live_rs::message::bud_property::{EqualizerType, Placement};

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
    pub batt_left: i8,
    pub batt_right: i8,
    pub battery_case: i8,
    pub placement_left: Placement,
    pub placement_right: Placement,
    pub equalizer_type: EqualizerType,
    pub touchpads_blocked: bool,
    pub noise_reduction: bool,
}

impl BudsInfo {
    pub fn new(stream: UnixStream) -> Self {
        Self {
            stream,
            batt_left: 0,
            batt_right: 0,
            battery_case: 0,
            placement_left: Placement::Undetected,
            placement_right: Placement::Undetected,
            equalizer_type: EqualizerType::Undetected,
            touchpads_blocked: false,
            noise_reduction: false,
        }
    }
}

impl std::fmt::Debug for BudsInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "batt_left: {}, batt_right: {}, batt_case: {}, placement_left: {:?}, placement_right: {:?}, equalizer_type: {:?}, touchpads_blocked: {}, noise_reduction: {}",
               self.batt_left, self.batt_right, self.battery_case, self.placement_left, self.placement_right, self.equalizer_type, self.touchpads_blocked, self.noise_reduction)
    }
}
