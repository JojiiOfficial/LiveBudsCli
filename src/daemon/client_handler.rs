use super::bud_connection::{BudsConnection, BudsInfo};
use async_mutex::Mutex;
use async_std::io::prelude::*;
use async_std::os::unix::net::UnixStream;
use galaxy_buds_live_rs::message::{
    extended_status_updated::ExtendedStatusUpdate, ids, status_updated::StatusUpdate, Message,
};
use std::{collections::HashMap, sync::Arc};

/// Shared data for informations about connected buds
pub struct ConnectionData {
    pub data: HashMap<String, BudsInfo>,
}

impl ConnectionData {
    pub fn new() -> Self {
        ConnectionData {
            data: HashMap::new(),
        }
    }

    pub fn get_first_device(&self) -> Option<&BudsInfo> {
        for (_, v) in &self.data {
            return Some(v);
        }
        None
    }

    pub fn get_first_stream(&self) -> &UnixStream {
        &self.get_first_device().unwrap().stream
    }
}

/// Read buds data
pub async fn handle_client(connection: BudsConnection, cd: Arc<Mutex<ConnectionData>>) {
    let mut stream = connection.socket.get_stream();
    let mut buffer = [0; 2048];

    loop {
        let r = stream.read(&mut buffer[..]).await;
        if let Err(_) = r {
            return;
        }

        // The received message from the buds
        let message = Message::new(&buffer[0..r.unwrap()]);

        let mut lock = cd.lock().await;
        let info = lock
            .data
            .entry(connection.addr.clone())
            .or_insert(BudsInfo::new(stream.clone()));

        if message.get_id() == ids::STATUS_UPDATED {
            let update: StatusUpdate = message.into();
            info.batt_left = update.battery_left;
            info.batt_right = update.battery_right;
            continue;
        }

        if message.get_id() == ids::EXTENDED_STATUS_UPDATED {
            let update: ExtendedStatusUpdate = message.into();
            info.batt_left = update.battery_left;
            info.batt_right = update.battery_right;
            continue;
        }
    }
}
