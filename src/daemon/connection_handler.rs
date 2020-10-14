use super::bluetooth;
use super::bud_connection::{BudsInfo, ConnectionEventInfo};
use super::buds_config::{BudsConfig, Config};
use super::client_handler;

use async_std::sync::Arc;
use async_std::sync::Mutex;

use std::collections::HashMap;
use std::sync::mpsc::Receiver;

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
        self.connected_devices.iter().any(|i| **i == *dev)
    }

    /// Add a device to the ConnHandler
    pub fn add_device(&mut self, dev: String) {
        self.connected_devices.push(dev);
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
        for (i, v) in self.connected_devices.iter().enumerate() {
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
        if addr.is_empty() {
            return self.get_first_device();
        }

        for v in self.data.values() {
            if v.inner.address == *addr {
                return Some(v);
            }
        }

        None
    }

    /// Get device mutable
    pub fn get_device_mut(&mut self, addr: &str) -> Option<&mut BudsInfo> {
        for (_, v) in self.data.iter_mut() {
            if v.inner.address == *addr {
                return Some(v);
            }
        }
        None
    }

    // Get the full address of a device
    pub fn get_device_address(&self, addr: &str) -> Option<String> {
        if addr.is_empty() {
            return self.get_first_device().map(|i| i.inner.address.clone());
        }

        self.get_device(addr).map(|i| i.inner.address.clone())
    }

    fn get_first_device(&self) -> Option<&BudsInfo> {
        self.data.iter().next().map(|(_, v)| v)
    }
}

/// run the connection handler
pub async fn run(
    rec: Receiver<ConnectionEventInfo>,
    cd: Arc<Mutex<ConnectionData>>,
    config: Arc<Mutex<Config>>,
) {
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

        // Set default config for (apparently) new device
        let mut cfg = config.lock().await;
        if !cfg.has_device_config(&i.addr) {
            cfg.set_device_config(BudsConfig::new(i.addr.clone()))
                .await
                .unwrap();
        }

        // Create a new buds connection task
        async_std::task::spawn(client_handler::handle_client(
            connection.unwrap(),
            Arc::clone(&connection_handler.get_connection_data()),
            Arc::clone(&config),
        ));
    }
}
