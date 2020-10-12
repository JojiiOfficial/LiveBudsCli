use async_std::os::unix::net::UnixStream;

/// An active connection to a pair of buds
#[derive(Debug, Clone)]
pub struct BudsConnection {
    pub addr: String,
    pub stream: UnixStream,
    pub fd: i32,
}

impl BudsConnection {
    pub async fn run(self) {}
}
