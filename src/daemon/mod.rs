mod bluetooth;
mod bud_connection;
mod connection_handler;
mod unix_socket;
mod utils;

use bud_connection::ConnectInfo;
use connection_handler::ConnectionData;
use std::sync::{mpsc, Arc, Mutex};

/// Starts the complete daemon
pub async fn run_daemon() {
    daemonize_self(); // Put into background

    let (conn_tx, conn_rx) = mpsc::channel::<ConnectInfo>();

    let arc = Arc::new(Mutex::new(ConnectionData::new()));

    async_std::task::spawn(connection_handler::run(conn_rx, Arc::clone(&arc)));
    async_std::task::spawn(unix_socket::run(Arc::clone(&arc)));
    bluetooth::run(conn_tx).await;
}

fn daemonize_self() {}
