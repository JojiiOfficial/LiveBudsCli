use super::super::super::buds_info::BudsInfo;
use galaxy_buds_rs::{message::extended_status_updated::ExtendedStatusUpdate, model::Model};

pub fn handle(update: ExtendedStatusUpdate, info: &mut BudsInfo) {
    // Update values from extended update
    update_extended_status(update, info);

    // Set ready after first extended status update
    if !info.inner.ready {
        info.inner.ready = true
    }
}

// Update a BudsInfo to the values of an extended_status_update
fn update_extended_status(update: ExtendedStatusUpdate, info: &mut BudsInfo) {
    info.inner.batt_left = update.battery_left;
    info.inner.batt_right = update.battery_right;
    info.inner.batt_case = update.battery_case;
    info.inner.placement_left = update.placement_left;
    info.inner.placement_right = update.placement_right;
    info.inner.equalizer_type = update.equalizer_type;
    info.inner.touchpads_blocked = update.touchpads_blocked;
    info.inner.noise_reduction = update.noise_reduction;
    info.inner.touchpad_option_left = update.touchpad_option_left;
    info.inner.touchpad_option_right = update.touchpad_option_right;

    // Ambient mode for buds
    if info.inner.model == Model::BudsPlus {
        info.inner.ambient_sound_volume = update.ambient_sound_volume as u8; // TODO make u8
        info.inner.ambient_sound_enabled = update.ambient_sound_enabled;
        info.inner.extra_high_ambient_volume = update.extra_high_ambient;
    }
}
