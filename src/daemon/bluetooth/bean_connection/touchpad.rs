#![allow(unused_variables)]

use std::time::SystemTime;

use super::super::super::unix_socket::bluetooth_commands;
use super::super::super::{buds_config::Config, buds_info::BudsInfo};
use super::super::bt_connection_listener::BudsConnection;

use async_std::sync::{Arc, Mutex};
use galaxy_buds_rs::message::{
    bud_property::{Side, TouchpadOption},
    touchpad_action::TouchAction,
};

const REQUIRED_TAP_DURATION: u8 = 2;

// Handle a status update
pub async fn handle(
    tap_info: TouchAction,
    info: &mut BudsInfo,
    config: &Arc<Mutex<Config>>,
    connection: &BudsConnection,
) -> bool {
    let early_exit;

    if info.inner.touchpads_blocked {
        // Separate config logic to keep cfg locked as short as possible
        early_exit = {
            // Lock the config
            let mut cfg = config.lock().await;

            // Load the (possibly changed) config values
            cfg.load().await.unwrap();

            let config = cfg.get_device_config(&connection.addr);
            if config.is_none() {
                true
            } else {
                let config = config.unwrap();
                if !config.hold_to_disconnect.unwrap_or(false) {
                    true
                } else {
                    false
                }
            }
        };
    } else {
        // Set early_exit to false if the tapped side is not set to Disconnect
        early_exit = match tap_info.side {
            Side::Left => info.inner.touchpad_option_left,
            Side::Right => info.inner.touchpad_option_right,
        } != TouchpadOption::Disconnect;
    }

    if tap_info.touch_count != 7 || early_exit {
        return false;
    }

    // We don't need that hold count crap if the tap-action is set to 'Disconnect' and touchpads
    // are enabled
    if !info.inner.touchpads_blocked {
        bluetooth_commands::change_connection_status(&connection.addr, false).await;
        info.reset_last_tp_update();
        return true;
    }

    // Reset Touchpad action counter
    if info
        .last_tp_update
        .elapsed()
        .map(|i| i.as_secs())
        .unwrap_or_default()
        > 5
        // Reset if values are too big (to prevent integer overflows in case the buds sends more
        // events than ususal and the user has lots of patience)
        || info.left_tp_hold_count + info.right_tp_hold_count > 20
    {
        info.reset_last_tp_update();
    }

    // Update hold count
    match tap_info.side {
        Side::Left => info.left_tp_hold_count += 1,
        Side::Right => info.right_tp_hold_count += 1,
    };
    info.last_tp_update = SystemTime::now();

    if info.left_tp_hold_count >= REQUIRED_TAP_DURATION
        && info.right_tp_hold_count >= REQUIRED_TAP_DURATION
    {
        // Disconnect
        bluetooth_commands::change_connection_status(&connection.addr, false).await;
        info.reset_last_tp_update();
        return true;
    }
    false
}
