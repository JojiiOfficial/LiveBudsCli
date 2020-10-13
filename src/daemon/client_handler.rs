use super::bud_connection::{BudsConnection, BudsInfo};
use async_std::io::prelude::*;
use galaxy_buds_live_rs::message::{
    self, extended_status_updated::ExtendedStatusUpdate, ids, status_updated::StatusUpdate,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

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

    pub fn data(&self) -> String {
        format!("{:?}", self.data)
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

        let num_bytes_read = r.unwrap();
        let buff = &buffer[0..num_bytes_read];
        let id = buff[3].to_be();
        let message = message::Message::new(buff);

        let mut lock = cd.lock().unwrap();
        let info = lock
            .data
            .entry(connection.addr.clone())
            .or_insert(BudsInfo::new());

        if id == ids::STATUS_UPDATED {
            let update: StatusUpdate = message.into();
            info.batt_left = update.battery_left;
            info.batt_right = update.battery_right;
            continue;
        }

        if id == ids::EXTENDED_STATUS_UPDATED {
            let update: ExtendedStatusUpdate = message.into();
            info.batt_left = update.battery_left;
            info.batt_right = update.battery_right;
            continue;
        }
    }
}
