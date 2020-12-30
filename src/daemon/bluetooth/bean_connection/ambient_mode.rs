use super::super::super::buds_info::BudsInfo;
use galaxy_buds_rs::message::ambient_mode::AmbientModeUpdated;

pub fn handle(update: AmbientModeUpdated, info: &mut BudsInfo) {
    info.inner.ambient_sound_enabled = update.ambient_mode;
}
