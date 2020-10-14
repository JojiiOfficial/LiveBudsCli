mod bluetooth;
pub mod bud_connection;
mod buds_config;
mod client_handler;
mod connection_handler;
pub mod unix_request_handler;
mod unix_socket;
pub mod utils;

use async_std::sync::Mutex;
use bud_connection::ConnectionEventInfo;
use connection_handler::ConnectionData;

use std::sync::{mpsc, Arc};

/// Starts the complete daemon
pub async fn run_daemon(p: String) {
    daemonize_self(); // Put into background

    // Exchange connection events between bluetooth and connection handler
    let (conn_tx, conn_rx) = mpsc::channel::<ConnectionEventInfo>();

    // Exchanging Buds data between unix socket and the buds listener
    let connection_data = Arc::new(Mutex::new(ConnectionData::new()));

    // Config setup
    let config = buds_config::Config::new()
        .await
        .expect("Couldn't read config");

    let config = Arc::new(Mutex::new(config));

    // Run connection handler
    async_std::task::spawn(connection_handler::run(
        conn_rx,
        Arc::clone(&connection_data),
        Arc::clone(&config),
    ));

    // Run unix socket
    async_std::task::spawn(unix_socket::run(
        p,
        Arc::clone(&connection_data),
        Arc::clone(&config),
    ));

    // Run bluetooth listener
    bluetooth::run(conn_tx).await;
}

fn daemonize_self() {}
