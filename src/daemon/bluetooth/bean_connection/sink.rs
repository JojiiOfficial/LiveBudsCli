use super::utils;
use crate::daemon::buds_info::BudsInfo;

use galaxy_buds_live_rs::message::status_updated::StatusUpdate;

#[cfg(feature = "pulse-sink")]
use pulsectl::controllers::{types::DeviceInfo, DeviceControl, SinkController};

// Change the default output sink to earbuds if they ain't yet
#[cfg(feature = "pulse-sink")]
pub fn make_sink_default(info: &BudsInfo) -> Option<()> {
    let mut handler = SinkController::create();

    if !is_default(&mut handler, &info).unwrap_or(true) {
        // Buds are not set to default
        let device = get_bt_sink(&mut handler, &info)?;
        handler.set_default_device(&device.name?).ok()?;
    }

    None
}

pub fn handle_low_battery(update: &StatusUpdate, info: &mut BudsInfo) {
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

// Return true if Earbuds are currently the default output device
#[cfg(feature = "pulse-sink")]
pub fn is_default(handler: &mut SinkController, info: &BudsInfo) -> Option<bool> {
    let device = get_bt_sink(handler, info)?;
    let default_device = handler.get_default_device().ok()?;
    Some(device.name.as_ref()? == default_device.name.as_ref()?)
}

// Change the default output sink to fallback if buds are placed into the case
#[cfg(feature = "pulse-sink")]
pub fn fallback_to_sink(info: &mut BudsInfo, update: &StatusUpdate) -> Option<()> {
    let was_in_case = utils::is_placed_state(info.inner.placement_left, info.inner.placement_right);
    let is_in_case = utils::is_placed_state(update.placement_left, update.placement_right);

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
            utils::try_play();
            info.inner.paused_music_earlier = false;
        }
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
