mod bluetooth;
mod unix_socket;
mod utils;

/// Starts the complete daemon
pub async fn run_daemon() {
    async_std::task::spawn(bluetooth::run());
    unix_socket::run().await;
}
