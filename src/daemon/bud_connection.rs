use async_std::os::unix::net::UnixStream;
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

pub struct BudsInfo {
    pub stream: UnixStream,
    pub batt_left: i8,
    pub batt_right: i8,
}

impl BudsInfo {
    pub fn new(stream: UnixStream) -> Self {
        Self {
            batt_left: 0,
            batt_right: 0,
            stream,
        }
    }
}

impl std::fmt::Debug for BudsInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "b_r: {}, b_l: {}", self.batt_left, self.batt_right)
    }
}
