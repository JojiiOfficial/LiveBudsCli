use super::super::super::buds_info::BudsInfo;
use galaxy_buds_rs::message::{bud_property::Side, debug::GetAllData};

pub fn handle(update: GetAllData, info: &mut BudsInfo) {
    // Update values from extended update
    update_data(update, info);

    // Set ready after first extended status update
    if !info.inner.ready {
        info.inner.ready = true
    }
}

// Update a BudsInfo to the values of an extended_status_update
fn update_data(update: GetAllData, info: &mut BudsInfo) {
    info.inner.debug.voltage_left = update.get_adc_vcell(Side::Left);
    info.inner.debug.voltage_right = update.get_adc_vcell(Side::Right);
    info.inner.debug.temperature_left = update.get_thermistor(Side::Left);
    info.inner.debug.temperature_right = update.get_thermistor(Side::Right);
    info.inner.debug.current_left = update.get_adc_current(Side::Left);
    info.inner.debug.current_right = update.get_adc_current(Side::Right);
}
