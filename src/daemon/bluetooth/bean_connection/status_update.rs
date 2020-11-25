use std::process::exit;

use super::super::super::buds_config::{BudsConfig, Config};
use super::super::super::buds_info::BudsInfo;
use super::super::bt_connection_listener::BudsConnection;
use super::sink;
use super::utils;

use async_std::sync::{Arc, Mutex};
use galaxy_buds_live_rs::message::status_updated::StatusUpdate;

#[cfg(feature = "pulse-sink")]
use pulsectl::controllers::SinkController;

// Handle a status update
pub async fn handle(
    update: StatusUpdate,
    info: &mut BudsInfo,
    config: &Arc<Mutex<Config>>,
    connection: &BudsConnection,
) {
    // Lock the config
    let mut cfg = config.lock().await;

    // Load the (possibly changed) config values
    if let Err(err) = cfg.load().await {
        eprintln!("{}", err);
        exit(1);
    }

    // Check if current device has a config entry
    if let Some(config) = cfg.get_device_config(&connection.addr) {
        // Play/Pause audio
        if config.auto_play() || config.auto_pause() || config.smart_sink() {
            handle_auto_music(&update, info, &config);
        }

        // handle desktop notification
        if config.low_battery_notification() {
            sink::handle_low_battery(&update, info);
        }

        // Fallback to next available sink if buds
        // get placed into the case
        #[cfg(feature = "pulse-sink")]
        if config.smart_sink() {
            sink::fallback_to_sink(info, &update);
        }
    }

    // Update the local status of the buds
    update_status(&update, info);
}

/// Handle automatically pausing/playing music on earbuds wearing statu changes
fn handle_auto_music(update: &StatusUpdate, info: &mut BudsInfo, config: &BudsConfig) {
    let is_wearing = utils::is_wearing_state(update.placement_left, update.placement_right);

    let was_wearing =
        utils::is_wearing_state(info.inner.placement_left, info.inner.placement_right);

    let was_some_wearing =
        utils::is_some_wearing_state(info.inner.placement_left, info.inner.placement_right);

    let is_some_wearing_state =
        utils::is_some_wearing_state(update.placement_left, update.placement_right);

    #[cfg(feature = "pulse-sink")]
    let mut handler = SinkController::create();

    // True if put buds on
    if !was_wearing && is_wearing {
        // Auto sink change
        #[cfg(feature = "pulse-sink")]
        if config.smart_sink() {
            sink::make_sink_default(&info);
        }

        // Don't do music actions if buds aren't default device
        #[cfg(feature = "pulse-sink")]
        if !sink::is_default(&mut handler, &info).unwrap_or(true) {
            return;
        }

        // Auto resume
        if config.auto_play() {
            if info.inner.paused_music_earlier {
                utils::try_play();
                info.inner.paused_music_earlier = false;
            }
        }
    } else if !is_some_wearing_state && was_some_wearing {
        // True if take the buds off

        // Don't do music actions if buds aren't default device
        #[cfg(feature = "pulse-sink")]
        if !sink::is_default(&mut handler, &info).unwrap_or(true) {
            return;
        }

        if config.auto_pause() {
            // Auto pause music
            if utils::try_pause() {
                info.inner.paused_music_earlier = true;
            }
        }
    }
}

// Update a BudsInfo to the values of an extended_status_update
fn update_status(update: &StatusUpdate, info: &mut BudsInfo) {
    info.inner.batt_left = update.battery_left;
    info.inner.batt_right = update.battery_right;
    info.inner.batt_case = update.battery_case;
    info.inner.placement_left = update.placement_left;
    info.inner.placement_right = update.placement_right;
}
