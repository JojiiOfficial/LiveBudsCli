#![allow(unused_variables)]

use super::super::super::unix_socket::bluetooth_commands;
use super::super::super::{buds_config::Config, buds_info::BudsInfo};
use super::super::bt_connection_listener::BudsConnection;

use async_std::sync::{Arc, Mutex};
use galaxy_buds_rs::message::{bud_property::Side, touchpad_action::TouchAction};

const REQUIRED_TAP_DURATION: u8 = 3;

// Handle a status update
pub async fn handle(
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
    if let Some(config) = cfg.get_device_config(&connection.addr) {
        if !config.hold_to_disconnect.unwrap_or(false) {
            return;
        }
    } else {
        return;
    }

    if tap_info.touch_count != 7 {
        return;
    }

    // Update hold count
    match tap_info.side {
        Side::Left => info.left_tp_hold_count += 1,
        Side::Right => info.right_tp_hold_count += 1,
    };

    if info.left_tp_hold_count >= REQUIRED_TAP_DURATION
        && info.right_tp_hold_count >= REQUIRED_TAP_DURATION
    {
        bluetooth_commands::change_connection_status(&connection.addr, false).await;

        info.left_tp_hold_count = 0;
        info.right_tp_hold_count = 0;
    }
}
