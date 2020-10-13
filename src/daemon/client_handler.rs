use super::bud_connection::{BudsConnection, BudsInfo};
use super::buds_config::Config;
use super::connection_handler::ConnectionData;

use async_std::io::prelude::*;
use async_std::sync::Mutex;
use galaxy_buds_live_rs::message::{
    extended_status_updated::ExtendedStatusUpdate, ids, status_updated::StatusUpdate, Message,
};

use std::sync::Arc;

/// Read buds data
pub async fn handle_client(
    connection: BudsConnection,
    cd: Arc<Mutex<ConnectionData>>,
    config: Arc<Mutex<Config>>,
) {
    let mut stream = connection.socket.get_stream();
    let mut buffer = [0; 2048];

    loop {
        let bytes_read = match stream.read(&mut buffer[..]).await {
            Ok(v) => v,
            Err(_) => return,
        };

        // The received message from the buds
        let message = Message::new(&buffer[0..bytes_read]);

        let mut lock = cd.lock().await;
        let info = lock
            .data
            .entry(connection.addr.clone())
            .or_insert_with(|| BudsInfo::new(stream.clone(), &connection.addr));

        if message.get_id() == ids::STATUS_UPDATED {
            update_status(&message.into(), info);
            continue;
        }

        if message.get_id() == ids::EXTENDED_STATUS_UPDATED {
            update_extended_status(&message.into(), info);
            continue;
        }
    }
}

// Update a BudsInfo to the values of an extended_status_update
fn update_extended_status(update: &ExtendedStatusUpdate, info: &mut BudsInfo) {
    info.batt_left = update.battery_left;
    info.batt_right = update.battery_right;
    info.batt_case = update.battery_case;
    info.placement_left = update.placement_left;
    info.placement_right = update.placement_right;
    info.equalizer_type = update.equalizer_type;
    info.touchpads_blocked = update.touchpads_blocked;
    info.noise_reduction = update.noise_reduction;
}

// Update a BudsInfo to the values of an extended_status_update
fn update_status(update: &StatusUpdate, info: &mut BudsInfo) {
    info.batt_left = update.battery_left;
    info.batt_right = update.battery_right;
    info.batt_case = update.battery_case;
    info.placement_left = update.placement_left;
    info.placement_right = update.placement_right;
}
