mod bluetooth;
mod buds_config;
pub mod buds_info;
pub mod unix_socket;
pub mod utils;

use async_std::sync::Mutex;
use bluetooth::rfcomm_connector::ConnectionData;

use std::{
    sync::{mpsc, Arc},
    thread,
};

use self::bluetooth::rfcomm_connector::ConnectionEventData;

/// Starts the complete daemon
pub async fn run_daemon(p: String) {
    // Exchange connection events between bluetooth and connection handler
    let (conn_tx, conn_rx) = mpsc::channel::<ConnectionEventData>();

    // Exchanging Buds data between unix socket and the buds listener
    let connection_data = Arc::new(Mutex::new(ConnectionData::new()));

    // Config setup
    let config = Arc::new(Mutex::new(
        buds_config::Config::new()
            .await
            .expect("Couldn't read config"),
    ));

    // Run Unix socket listener
    async_std::task::spawn(unix_socket::socket::run(
        p,
        Arc::clone(&connection_data),
        Arc::clone(&config),
    ));

    // Run connection handler
    async_std::task::spawn(bluetooth::rfcomm_connector::run(
        conn_rx,
        Arc::clone(&connection_data),
        Arc::clone(&config),
    ));

    // Run bluetooth listener
    thread::Builder::new()
        .stack_size(1 * 1024 * 1024)
        .spawn(|| {
            bluetooth::bt_connection_listener::run(conn_tx);
        })
        .expect("can't spawn thread")
        .join()
        .expect("Thread spawning failed");
}
