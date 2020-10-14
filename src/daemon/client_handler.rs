use super::bud_connection::{BudsConnection, BudsInfo};
use super::buds_config::{BudsConfig, Config};
use super::connection_handler::ConnectionData;

use async_std::io::prelude::*;
use async_std::sync::Mutex;
use galaxy_buds_live_rs::message::{
    bud_property::Placement, extended_status_updated::ExtendedStatusUpdate, ids,
    status_updated::StatusUpdate, Message,
};
use mpris::PlayerFinder;

use std::sync::Arc;

/// Read buds data
pub async fn handle_client(
    connection: BudsConnection,
    cd: Arc<Mutex<ConnectionData>>,
    config: Arc<Mutex<Config>>,
) {
    let mut stream = connection.socket.get_stream();
    let mut buffer = [0; 2048];
    let has_finder = PlayerFinder::new().is_ok();

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
            let update = message.into();

            // Handle auto resume music
            {
                if has_finder {
                    let mut cfg = config.lock().await;
                    cfg.load().await.unwrap();
                    if let Some(config) = cfg.get_device_config(&connection.addr) {
                        if config.auto_resume_music || config.auto_pause_music {
                            handle_auto_music(&update, info, &config);
                        }
                    }
                }
            }

            update_status(&update, info);
            continue;
        }

        if message.get_id() == ids::EXTENDED_STATUS_UPDATED {
            update_extended_status(&message.into(), info);
            continue;
        }
    }
}

/// Handle automatically pausing/playing music on earbuds wearing statu changes
fn handle_auto_music(update: &StatusUpdate, info: &BudsInfo, config: &BudsConfig) {
    let is_wearing = is_wearing_state(update.placement_left, update.placement_right);
    let was_wearing = is_wearing_state(info.placement_left, info.placement_right);

    let was_some_wearing = is_some_wearing_state(info.placement_left, info.placement_right);
    let is_not_wearing = is_absolute_not_wearing(update.placement_left, update.placement_right);

    // Resume music if old wearing state
    // is false and update's wearing state is true
    if !was_wearing && is_wearing {
        // Auto resume
        if !config.auto_resume_music {
            return;
        }

        if let Ok(finder) = PlayerFinder::new() {
            let player = finder.find_active();
            if let Ok(player) = player {
                player.play().unwrap();
            }
        }
    } else if is_not_wearing && was_some_wearing {
        // Auto pause music
        if !config.auto_pause_music {
            return;
        }

        if let Ok(finder) = PlayerFinder::new() {
            let player = finder.find_active();
            if let Ok(player) = player {
                player.pause().unwrap();
            }
        }
    }
}

fn is_wearing_state(left: Placement, right: Placement) -> bool {
    left == Placement::Ear && right == Placement::Ear
}

fn is_some_wearing_state(left: Placement, right: Placement) -> bool {
    left == Placement::Ear && right == Placement::Ear
}

fn is_absolute_not_wearing(left: Placement, right: Placement) -> bool {
    left != Placement::Ear && right != Placement::Ear
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
