use super::super::super::buds_info::BudsInfo;
use galaxy_buds_rs::message::anc_updated::AncModeUpdated;

pub fn handle(update: AncModeUpdated, info: &mut BudsInfo) {
    info.inner.noise_reduction = update.anc_enabled;
}
