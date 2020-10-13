use super::bluetooth;
use super::bud_connection::{BudsInfo, ConnectionEventInfo};
use super::client_handler;

use async_std::sync::Mutex;

use std::collections::HashMap;
use std::sync::{mpsc::Receiver, Arc};

/// The connection handler keeps track of
/// all connected devices and its status
pub struct ConnHandler {
    connected_devices: Vec<String>,
    connection_data: Arc<Mutex<ConnectionData>>,
}

impl ConnHandler {
    /// Create a new Connection handler
    pub fn new(cd: Arc<Mutex<ConnectionData>>) -> Self {
        ConnHandler {
            connected_devices: Vec::new(),
            connection_data: cd,
        }
    }

    /// Get an Arc::Clone of the ConnectionData
    pub fn get_connection_data(&self) -> Arc<Mutex<ConnectionData>> {
        Arc::clone(&self.connection_data)
    }

    /// Check whether a given device is connected or not
    pub fn has_device(&self, dev: &str) -> bool {
        self.connected_devices
            .as_slice()
            .into_iter()
            .find(|i| **i == *dev)
            .is_some()
    }

    /// Add a device to the ConnHandler
    pub fn add_device(&mut self, dev: String) {
        self.connected_devices.push(dev.clone());
    }

    /// Remove a device from the ConnHandler
    pub async fn remove_device(&mut self, dev: &str) {
        let pos = self.get_item_pos(dev);
        if pos.is_none() {
            return;
        }

        self.connection_data.lock().await.data.remove(dev);
        self.connected_devices.remove(pos.unwrap());
    }

    /// Get the position of a device in the ConnHandler device vector
    pub fn get_item_pos(&self, dev: &str) -> Option<usize> {
        for (i, v) in self.connected_devices.as_slice().into_iter().enumerate() {
            if *v == *dev {
                return Some(i);
            }
        }
        None
    }
}

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

    /// Returns a device by its address. If no address is set,
    /// the first device gets returned
    pub fn get_device(&self, addr: &str) -> Option<&BudsInfo> {
        if addr.len() == 0 {
            return self.get_first_device();
        }

        for (_, v) in &self.data {
            if v.address == *addr {
                return Some(v);
            }
        }

        None
    }

    fn get_first_device(&self) -> Option<&BudsInfo> {
        for (_, v) in &self.data {
            return Some(v);
        }

        None
    }
}

/// run the connection handler
pub async fn run(rec: Receiver<ConnectionEventInfo>, cd: Arc<Mutex<ConnectionData>>) {
    let mut connection_handler = ConnHandler::new(cd);

    for i in rec {
        if !i.connected {
            // remove connection
            connection_handler.remove_device(i.addr.as_str()).await;
            continue;
        }

        // Ignore already connected devices
        if connection_handler.has_device(i.addr.as_str()) {
            continue;
        }

        // Connect to the RFCOMM interface of the buds
        let connection = bluetooth::connect_rfcomm(&i.addr);
        if let Err(err) = connection {
            eprintln!("Error connecting to rfcomm:{:?}", err);
            continue;
        }

        println!("Connected successfully to Buds live!");

        // Add device to the connection handler
        connection_handler.add_device(i.addr.to_owned());

        // Create a new buds connection task
        async_std::task::spawn(client_handler::handle_client(
            connection.unwrap(),
            Arc::clone(&connection_handler.get_connection_data()),
        ));
    }
}
