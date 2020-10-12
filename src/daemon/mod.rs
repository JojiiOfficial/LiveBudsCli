mod bluetooth;
mod bud_connection;
mod connection_handler;
mod unix_socket;
mod utils;

use bud_connection::BudsConnection;
use connection_handler::ConnectionHandler;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Starts the complete daemon
pub async fn run_daemon() {
    daemonize_self(); // Put into background

    let connections = Arc::new(Mutex::new(ConnectionHandler::new()));

    async_std::task::spawn(bluetooth::run());
    unix_socket::run().await;
}

fn daemonize_self() {}
