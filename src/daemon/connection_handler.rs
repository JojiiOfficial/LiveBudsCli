use super::bud_connection::{BudsConnection, ConnectInfo};
use super::client_handler::{self, ConnectionData};

use async_std::sync::Mutex;
use bluetooth_serial_port_async::{BtAddr, BtProtocol, BtSocket};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::{error::Error, str::FromStr};

pub struct ConnHandler {
    connected_devices: Vec<String>,
    connection_data: Arc<Mutex<ConnectionData>>,
}

impl ConnHandler {
    pub fn new(cd: Arc<Mutex<ConnectionData>>) -> Self {
        ConnHandler {
            connected_devices: Vec::new(),
            connection_data: cd,
        }
    }

    pub fn get_connection_data(&self) -> Arc<Mutex<ConnectionData>> {
        Arc::clone(&self.connection_data)
    }

    pub fn has_device(&self, dev: &str) -> bool {
        self.connected_devices
            .as_slice()
            .into_iter()
            .find(|i| **i == *dev)
            .is_some()
    }

    pub fn add_device(&mut self, dev: String) {
        self.connected_devices.push(dev.clone());
    }

    pub async fn remove_device(&mut self, dev: &str) {
        let pos = self.get_item_pos(dev);
        if pos.is_none() {
            return;
        }

        self.connection_data.lock().await.data.remove(dev);
        self.connected_devices.remove(pos.unwrap());
    }

    pub fn get_item_pos(&self, dev: &str) -> Option<usize> {
        for (i, v) in self.connected_devices.as_slice().into_iter().enumerate() {
            if *v == *dev {
                return Some(i);
            }
        }
        None
    }
}

/// run the connection handler
pub async fn run(rec: Receiver<ConnectInfo>, cd: Arc<Mutex<ConnectionData>>) {
    let mut connection_handler = ConnHandler::new(cd);

    for i in rec {
        if !i.connected {
            // remove connection
            connection_handler.remove_device(i.addr.as_str()).await;
            continue;
        }

        if connection_handler.has_device(i.addr.as_str()) {
            println!("dev already connected!");
            continue;
        }

        let connection = connect_rfcomm(&i.addr);
        if let Err(err) = connection {
            eprintln!("Error connecting to rfcomm:{:?}", err);
            continue;
        }

        println!("Connected successfully to Buds live!");
        connection_handler.add_device(i.addr.to_owned());

        async_std::task::spawn(client_handler::handle_client(
            connection.unwrap(),
            Arc::clone(&connection_handler.get_connection_data()),
        ));
    }
}

/// Connect to buds live via rfcomm proto
fn connect_rfcomm<S: AsRef<str>>(addr: S) -> Result<BudsConnection, Box<dyn Error>> {
    let mut socket = BtSocket::new(BtProtocol::RFCOMM)?;
    let address = BtAddr::from_str(addr.as_ref()).unwrap();
    socket.connect(&address)?;
    let fd = socket.get_fd();

    Ok(BudsConnection {
        addr: addr.as_ref().to_owned(),
        socket,
        fd,
    })
}
