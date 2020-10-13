mod bluetooth;
mod bud_connection;
mod client_handler;
mod connection_handler;
mod unix_socket;
mod utils;

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
    let arc = Arc::new(Mutex::new(ConnectionData::new()));

    async_std::task::spawn(connection_handler::run(conn_rx, Arc::clone(&arc)));
    async_std::task::spawn(unix_socket::run(p, Arc::clone(&arc)));
    bluetooth::run(conn_tx).await;
}

fn daemonize_self() {}
