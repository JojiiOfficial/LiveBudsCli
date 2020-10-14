use super::bluetooth::BudsConnection;
use super::buds_config::{BudsConfig, Config};
use super::buds_info::BudsInfo;
use super::connection_handler::ConnHandler;

use async_std::io::prelude::*;
use async_std::sync::Mutex;
use galaxy_buds_live_rs::message::{
    bud_property::Placement, extended_status_updated::ExtendedStatusUpdate, ids,
    status_updated::StatusUpdate, Message,
};
use mpris::PlayerFinder;
use notify_rust::Notification;

use std::sync::Arc;

/// Read buds data
pub async fn handle_client(
    connection: BudsConnection,
    config: Arc<Mutex<Config>>,
    ch: Arc<Mutex<ConnHandler>>,
) {
    let mut stream = connection.socket.get_stream();
    let mut buffer = [0; 2048];

    loop {
        let bytes_read = match stream.read(&mut buffer[..]).await {
            Ok(v) => v,
            Err(_) => {
                let mut c = ch.lock().await;
                c.remove_device(connection.addr.as_str()).await;

                println!("exited handle_client");
                return;
            }
        };

        // The received message from the buds
        let message = Message::new(&buffer[0..bytes_read]);

        let connection_handler = ch.lock().await;
        let mut lock = connection_handler.connection_data.lock().await;

        let info = lock
            .data
            .entry(connection.addr.clone())
            .or_insert_with(|| BudsInfo::new(stream.clone(), &connection.addr));

        if message.get_id() == ids::STATUS_UPDATED {
            let update = message.into();

            // Handle auto resume music and battery notifications
            {
                let mut cfg = config.lock().await;
                cfg.load().await.unwrap();

                if let Some(config) = cfg.get_device_config(&connection.addr) {
                    if config.auto_resume_music || config.auto_pause_music {
                        handle_auto_music(&update, info, &config);
                    }

                    if config.low_battery_notification {
                        handle_low_battery(&update, info);
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
    let was_wearing = is_wearing_state(info.inner.placement_left, info.inner.placement_right);

    let was_some_wearing = is_wearing_state(info.inner.placement_left, info.inner.placement_right);
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

fn is_absolute_not_wearing(left: Placement, right: Placement) -> bool {
    left != Placement::Ear && right != Placement::Ear
}

fn handle_low_battery(update: &StatusUpdate, info: &mut BudsInfo) {
    let l_batt = update.battery_left;
    let r_batt = update.battery_right;

    // Reset battery notify lock
    if l_batt > 30 && r_batt > 30 && info.inner.did_battery_notify {
        info.inner.did_battery_notify = false;
        return;
    }

    if info.inner.did_battery_notify {
        return;
    }

    if l_batt < 20 || r_batt < 20 {
        info.inner.did_battery_notify = true;

        Notification::new()
            .summary("Buds Live battery low")
            .body(
                format!(
                    "The battery of your Galaxy buds live is pretty low: (L: {}%, R: {}%)",
                    l_batt, r_batt
                )
                .as_str(),
            )
            .icon("battery")
            .show()
            .unwrap();
    }
}

// Update a BudsInfo to the values of an extended_status_update
fn update_extended_status(update: &ExtendedStatusUpdate, info: &mut BudsInfo) {
    info.inner.batt_left = update.battery_left;
    info.inner.batt_right = update.battery_right;
    info.inner.batt_case = update.battery_case;
    info.inner.placement_left = update.placement_left;
    info.inner.placement_right = update.placement_right;
    info.inner.equalizer_type = update.equalizer_type;
    info.inner.touchpads_blocked = update.touchpads_blocked;
    info.inner.noise_reduction = update.noise_reduction;
}

// Update a BudsInfo to the values of an extended_status_update
fn update_status(update: &StatusUpdate, info: &mut BudsInfo) {
    info.inner.batt_left = update.battery_left;
    info.inner.batt_right = update.battery_right;
    info.inner.batt_case = update.battery_case;
    info.inner.placement_left = update.placement_left;
    info.inner.placement_right = update.placement_right;
}
