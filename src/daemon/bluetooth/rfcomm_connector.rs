use crate::daemon::bluetooth::bt_connection_listener::SharedStream;

use super::super::buds_config::{BudsConfig, Config};
use super::super::buds_info::BudsInfo;
use super::bean_connection;
use super::bt_connection_listener::BudsConnection;

use bluer::Address;
use galaxy_buds_rs::model::Model;
use log::info;
use tokio::sync::Mutex;

use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

/// The connection handler keeps track of
/// all connected devices and its status
pub struct ConnHandler {
    connected_devices: Vec<Address>,
    pub connection_data: Arc<Mutex<ConnectionData>>,
}

impl ConnHandler {
    /// Create a new Connection handler
    pub fn new(cd: Arc<Mutex<ConnectionData>>) -> Self {
        ConnHandler {
            connected_devices: Vec::new(),
            connection_data: cd,
        }
    }

    /// Check whether a given device is connected or not
    pub fn has_device(&self, dev: &Address) -> bool {
        self.connected_devices.contains(dev)
    }

    /// Add a device to the ConnHandler
    pub fn add_device(&mut self, dev: Address) {
        self.connected_devices.push(dev);
    }

    /// Remove a device from the ConnHandler
    pub async fn remove_device(&mut self, dev: &Address) {
        self.connection_data.lock().await.data.remove(dev);

        let pos = self.get_item_pos(dev);
        if pos.is_none() {
            return;
        }

        self.connected_devices.remove(pos.unwrap());
    }

    /// Get the position of a device in the ConnHandler device vector
    pub fn get_item_pos(&self, dev: &Address) -> Option<usize> {
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
    pub data: HashMap<Address, BudsInfo>,
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
    pub async fn get_device_address(
        &self,
        addr: &str,
        config: &Arc<Mutex<Config>>,
    ) -> Option<String> {
        if addr.is_empty() {
            if self.get_device_count() == 0 {
                if let Some(dev) = config.lock().await.get_default_device() {
                    return Some(dev.address.clone());
                } else {
                    return None;
                }
            }
            return self.get_first_device().map(|i| i.inner.address.clone());
        }

        let device = self.get_device(addr)?;

        if !device.inner.ready {
            return None;
        }

        Some(device.inner.address.clone())
    }

    /// Get count of connected devices
    pub fn get_device_count(&self) -> usize {
        self.data
            .iter()
            .find(|(_, item)| item.inner.ready)
            .iter()
            .count()
    }

    fn get_first_device(&self) -> Option<&BudsInfo> {
        self.data.iter().next().map(|(_, v)| v)
    }
}

/// run the connection handler
pub async fn run(
    rec: Receiver<ConnectionEventData>,
    cd: Arc<Mutex<ConnectionData>>,
    config: Arc<Mutex<Config>>,
) {
    let conn_handler = Arc::new(Mutex::new(ConnHandler::new(cd)));

    for connection_event in rec {
        // We must drop the connection handler guard to prevent locking the receiver of connected devices.
        {
            let mut connection_handler = conn_handler.lock().await;

            // Ignore already connected devices
            if connection_handler.has_device(&connection_event.address) {
                continue;
            }

            // Add device to the connection handler
            connection_handler.add_device(connection_event.address.clone());
        }

        info!("Connected successfully to {}", connection_event.model);

        // Set default config for (apparently) new device
        {
            let mut cfg = config.lock().await;
            let address_str = connection_event.address.to_string();
            if !cfg.has_device_config(&address_str) {
                cfg.set_device_config(BudsConfig::new(address_str))
                    .await
                    .unwrap();
            }
        }

        let connection = BudsConnection {
            addr: connection_event.address,
            stream: connection_event.stream,
        };

        // Create a new buds connection task
        tokio::task::spawn(bean_connection::listener::start_listen(
            connection,
            config.clone(),
            conn_handler.clone(),
            connection_event.model,
        ))
        .await
        .unwrap();
    }
}

#[derive(Debug)]
pub struct ConnectionEventData {
    pub address: Address,
    pub stream: SharedStream,
    pub model: Model,
}
