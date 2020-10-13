mod bluetooth;
mod bud_connection;
mod connection_handler;
mod unix_socket;
mod utils;

use std::sync::mpsc;

/// Starts the complete daemon
pub async fn run_daemon() {
    daemonize_self(); // Put into background

    let (conn_tx, conn_rx) = mpsc::channel::<String>();

    async_std::task::spawn(connection_handler::run(conn_rx));
    async_std::task::spawn(unix_socket::run());
    bluetooth::run(conn_tx).await;
}

fn daemonize_self() {}
