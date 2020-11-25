use std::process::exit;

use super::super::super::buds_config::{BudsConfig, Config};
use super::super::super::buds_info::BudsInfo;
use super::super::bt_connection_listener::BudsConnection;
use super::utils;

use async_std::sync::{Arc, Mutex};
use galaxy_buds_live_rs::message::status_updated::StatusUpdate;

#[cfg(feature = "pulse-sink")]
use pulsectl::controllers::{types::DeviceInfo, DeviceControl, SinkController};
use utils::{is_placed_state, try_play};

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
            handle_low_battery(&update, info);
        }

        // Fallback to next available sink if buds
        // get placed into the case
        if config.smart_sink() {
            fallback_to_sink(info, &update);
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
            make_sink_default(&info);
        }

        // Don't do music actions if buds aren't default device
        #[cfg(feature = "pulse-sink")]
        if !is_default(&mut handler, &info).unwrap_or(true) {
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
        if !is_default(&mut handler, &info).unwrap_or(true) {
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

// Return true if Earbuds are currently the default output device
#[cfg(feature = "pulse-sink")]
fn is_default(handler: &mut SinkController, info: &BudsInfo) -> Option<bool> {
    let device = get_bt_sink(handler, info)?;
    let default_device = handler.get_default_device().ok()?;
    Some(device.name.as_ref()? == default_device.name.as_ref()?)
}

// Change the default output sink to fallback if buds are placed into the case
#[cfg(feature = "pulse-sink")]
fn fallback_to_sink(info: &mut BudsInfo, update: &StatusUpdate) -> Option<()> {
    let was_in_case = is_placed_state(info.inner.placement_left, info.inner.placement_right);
    let is_in_case = is_placed_state(update.placement_left, update.placement_right);

    let mut handler = SinkController::create();

    if !was_in_case && is_in_case && is_default(&mut handler, &info)? {
        let devices = handler.list_devices().ok()?;
        let fb_device = devices
            .iter()
            .filter(|i| {
                !i.name
                    .as_ref()
                    .unwrap_or(&String::new())
                    .to_lowercase()
                    .contains(&info.inner.address.to_lowercase())
            })
            .next()?;

        println!("switch to device: {}", fb_device.name.as_ref().unwrap());
        handler.set_default_device(fb_device.name.as_ref()?).ok()?;

        // TODO make configurable
        // Continue music if stopped by putting into case
        if info.inner.paused_music_earlier {
            try_play();
            info.inner.paused_music_earlier = false;
        }
    }

    None
}

// Change the default output sink to earbuds if they ain't yet
#[cfg(feature = "pulse-sink")]
fn make_sink_default(info: &BudsInfo) -> Option<()> {
    let mut handler = SinkController::create();

    if !is_default(&mut handler, &info).unwrap_or(true) {
        // Buds are not set to default
        let device = get_bt_sink(&mut handler, &info)?;
        handler.set_default_device(&device.name?).ok()?;
    }

    None
}

#[cfg(feature = "pulse-sink")]
fn get_bt_sink(handler: &mut SinkController, info: &BudsInfo) -> Option<DeviceInfo> {
    let devices = handler.list_devices().ok()?;
    devices
        .iter()
        .find(|i| i.proplist.get_str("device.string").unwrap_or_default() == info.inner.address)
        .map(|i| i.to_owned())
}

fn handle_low_battery(update: &StatusUpdate, info: &mut BudsInfo) {
    let l_batt = update.battery_left;
    let r_batt = update.battery_right;

    // Reset battery notify lock
    if l_batt > 30 && r_batt > 30 && info.inner.did_battery_notify {
        info.inner.did_battery_notify = false;
        return;
    }

    // Check if already notified
    if info.inner.did_battery_notify {
        return;
    }

    // Display a notification below 20% (both have to be above 0%)
    if l_batt < 20 || r_batt < 20 && (l_batt * r_batt > 0) {
        info.inner.did_battery_notify = true;
        utils::get_desktop_notification(l_batt, r_batt)
            .show()
            .unwrap();
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
