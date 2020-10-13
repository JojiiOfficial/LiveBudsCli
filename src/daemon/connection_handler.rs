use super::bud_connection::{BudsConnection, ConnectInfo};
use super::client_handler::{self, ConnectionData};

use async_mutex::Mutex;
use bluetooth_serial_port_async::{BtAddr, BtProtocol, BtSocket};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::{error::Error, str::FromStr};

pub struct ConnHandler {
    connected_devices: Vec<String>,
}

impl ConnHandler {
    pub fn new() -> Self {
        ConnHandler {
            connected_devices: Vec::new(),
        }
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

    pub fn remove_device(&mut self, dev: String) {
        let pos = self.get_item_pos(dev);
        if pos.is_none() {
            return;
        }

        self.connected_devices.remove(pos.unwrap());
    }

    pub fn get_item_pos(&self, dev: String) -> Option<usize> {
        for (i, v) in self.connected_devices.as_slice().into_iter().enumerate() {
            if *v == dev {
                return Some(i);
            }
        }

        None
    }
}

/// run the connection handler
pub async fn run(rec: Receiver<ConnectInfo>, cd: Arc<Mutex<ConnectionData>>) {
    let mut connections = ConnHandler::new();

    for i in rec {
        if !i.connected {
            // remove connection
            connections.remove_device(i.addr);
            continue;
        }

        if connections.has_device(i.addr.as_str()) {
            println!("dev already connected!");
            continue;
        }

        let connection = connect_rfcomm(&i.addr);
        if let Err(err) = connection {
            eprintln!("Error connecting to rfcomm:{:?}", err);
            continue;
        }

        println!("Connected successfully to Buds live!");
        connections.add_device(i.addr.to_owned());

        async_std::task::spawn(client_handler::handle_client(
            connection.unwrap(),
            Arc::clone(&cd),
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
