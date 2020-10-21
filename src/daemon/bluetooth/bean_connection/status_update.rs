use super::super::super::buds_config::{BudsConfig, Config};
use super::super::super::buds_info::BudsInfo;
use super::super::bt_connection_listener::BudsConnection;
use super::utils;

use async_std::sync::{Arc, Mutex};
use galaxy_buds_live_rs::message::status_updated::StatusUpdate;

use pulsectl::controllers::{types::DeviceInfo, DeviceControl, SinkController};

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
    cfg.load().await.unwrap();

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
    }

    // Update the local status of the buds
    update_status(&update, info);
}

/// Handle automatically pausing/playing music on earbuds wearing statu changes
fn handle_auto_music(update: &StatusUpdate, info: &BudsInfo, config: &BudsConfig) {
    let is_wearing = utils::is_wearing_state(update.placement_left, update.placement_right);
    let was_wearing =
        utils::is_wearing_state(info.inner.placement_left, info.inner.placement_right);

    let was_some_wearing =
        utils::is_some_wearing_state(info.inner.placement_left, info.inner.placement_right);

    let is_absolute_not_wearing =
        utils::is_absolute_not_wearing(update.placement_left, update.placement_right);

    // True if put buds on
    if !was_wearing && is_wearing {
        // Auto sink change
        if config.smart_sink() {
            handle_sink_change(&info);
        }

        // Auto resume
        if config.auto_play() {
            utils::try_play();
        }
    } else if is_absolute_not_wearing && was_some_wearing {
        // True if take the buds off

        if config.auto_pause() {
            // Auto pause music
            utils::try_pause();
        }
    }
}

fn handle_sink_change(info: &BudsInfo) -> Option<()> {
    let mut handler = SinkController::create();
    let device = get_bt_sink(&mut handler, info)?;
    let default_device = handler.get_default_device().ok()?;

    if device.name.as_ref()? != default_device.name.as_ref()? {
        // Buds are not set to default
        handler.set_default_device(&device.name?).ok()?;
    }

    None
}

fn get_bt_sink(handler: &mut SinkController, info: &BudsInfo) -> Option<DeviceInfo> {
    let devices = handler.list_devices().ok()?;
    devices
        .iter()
        .find(|i| {
            i.proplist.get_str("device.string").unwrap_or("".to_owned()) == info.inner.address
        })
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
