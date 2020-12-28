#![allow(unused_variables)]

use super::super::super::{buds_config::Config, buds_info::BudsInfo};
use super::super::bt_connection_listener::BudsConnection;

use async_std::sync::{Arc, Mutex};
use galaxy_buds_rs::message::touchpad_action::TouchAction;

// Handle a status update
pub async fn handle_tap(
    tap_info: TouchAction,
    info: &mut BudsInfo,
    config: &Arc<Mutex<Config>>,
    connection: &BudsConnection,
) {
    // Lock the config
    let mut cfg = config.lock().await;

    // Load the (possibly changed) config values
    cfg.load().await.unwrap();

    // Check if current device has a config entry
    if let Some(config) = cfg.get_device_config(&connection.addr) {}
}
