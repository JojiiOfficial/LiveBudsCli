use bluetooth_serial_port_async::BtSocket;

/// An active connection to a pair of buds
#[derive(Debug)]
pub struct BudsConnection {
    pub addr: String,
    pub socket: BtSocket,
    pub fd: i32,
}

#[derive(Debug)]
pub struct ConnectInfo {
    pub addr: String,
    pub connected: bool,
}

impl ConnectInfo {
    pub fn new(addr: String, connected: bool) -> Self {
        ConnectInfo { addr, connected }
    }
}

#[derive(Debug, Default)]
pub struct BudsInfo {
    pub batt_left: i8,
    pub batt_right: i8,
}

impl BudsInfo {
    pub fn new() -> Self {
        Self::default()
    }
}
