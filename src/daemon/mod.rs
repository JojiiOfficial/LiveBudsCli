mod bluetooth;
mod buds_config;
pub mod buds_info;
pub mod unix_socket;
pub mod utils;

use bluetooth::rfcomm_connector::ConnectionData;
use futures::future::join_all;
use tokio::sync::Mutex;

use std::sync::{mpsc, Arc};

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
    let unix_jh = tokio::task::spawn(unix_socket::socket::run(
        p,
        connection_data.clone(),
        Arc::clone(&config),
    ));

    // Run connection handler
    let connhandler_jh = tokio::task::spawn(bluetooth::rfcomm_connector::run(
        conn_rx,
        Arc::clone(&connection_data),
        Arc::clone(&config),
    ));

    // Run bluetooth listener
    let bt_listener_jh = tokio::task::spawn(async {
        bluetooth::bt_connection_listener::run(conn_tx)
            .await
            .unwrap();
    });

    join_all([unix_jh, connhandler_jh, bt_listener_jh]).await;
}
