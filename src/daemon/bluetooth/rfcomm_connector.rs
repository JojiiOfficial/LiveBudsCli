use super::super::buds_config::{BudsConfig, Config};
use super::super::buds_info::BudsInfo;
use super::bean_connection;
use super::bt_connection_listener::BudsConnection;

use async_std::sync::Arc;
use async_std::sync::Mutex;
use bluetooth_serial_port_async::{BtAddr, BtProtocol, BtSocket};

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::mpsc::Receiver;

/// The connection handler keeps track of
/// all connected devices and its status
pub struct ConnHandler {
    connected_devices: Vec<String>,
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
    pub fn has_device(&self, dev: &str) -> bool {
        self.connected_devices.iter().any(|i| **i == *dev)
    }

    /// Add a device to the ConnHandler
    pub fn add_device(&mut self, dev: String) {
        self.connected_devices.push(dev);
    }

    /// Remove a device from the ConnHandler
    pub async fn remove_device(&mut self, dev: &str) {
        self.connection_data.lock().await.data.remove(dev);

        let pos = self.get_item_pos(dev);
        if pos.is_none() {
            return;
        }

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
    rec: Receiver<String>,
    cd: Arc<Mutex<ConnectionData>>,
    config: Arc<Mutex<Config>>,
) {
    let connection_handler = ConnHandler::new(cd);
    let arc_ch = Arc::new(Mutex::new(connection_handler));

    for i in rec {
        let mut connection_handler = arc_ch.lock().await;

        // Ignore already connected devices
        if connection_handler.has_device(i.as_str()) {
            continue;
        }

        // Connect to the RFCOMM interface of the buds
        let connection = connect_rfcomm(i.clone());
        if let Err(err) = connection {
            eprintln!("Error connecting to rfcomm: {:?}", err);
            continue;
        }

        println!("Connected successfully to Buds live!");

        // Add device to the connection handler
        connection_handler.add_device(i.to_owned());

        // Set default config for (apparently) new device
        {
            let mut cfg = config.lock().await;
            if !cfg.has_device_config(&i) {
                cfg.set_device_config(BudsConfig::new(i.clone()))
                    .await
                    .unwrap();
            }
        }

        // Create a new buds connection task
        async_std::task::spawn(bean_connection::listener::start_listen(
            connection.unwrap(),
            Arc::clone(&config),
            Arc::clone(&arc_ch),
        ));
    }
}

/// Connect to buds live via rfcomm proto
pub fn connect_rfcomm<S: AsRef<str>>(addr: S) -> Result<BudsConnection, String> {
    let mut socket = BtSocket::new(BtProtocol::RFCOMM).map_err(|e| e.to_string())?;
    let address = BtAddr::from_str(addr.as_ref()).unwrap();
    socket.connect(&address).map_err(|e| e.to_string())?;
    let fd = socket.get_fd();

    Ok(BudsConnection {
        addr: addr.as_ref().to_owned(),
        socket,
        fd,
    })
}
