use super::bud_connection::{BudsConnection, ConnectInfo};

use async_std::io::prelude::*;
use bluetooth_serial_port_async::{BtAddr, BtProtocol, BtSocket};

use std::marker::Send;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::{error::Error, str::FromStr};

pub struct ConnHandler {
    connected_devices: Vec<String>,
}

unsafe impl Send for ConnHandler {}

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
pub async fn run(rec: Receiver<ConnectInfo>) {
    let mut connections = ConnHandler::new();
    let cd = ConnectionData::new();

    let arc = Arc::new(Mutex::new(cd));

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

        async_std::task::spawn(handle_client(connection.unwrap(), Arc::clone(&arc)));
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

struct ConnectionData {
    msg_count: Vec<usize>,
}

impl ConnectionData {
    fn new() -> Self {
        ConnectionData {
            msg_count: Vec::new(),
        }
    }
}

/// Read buds data
async fn handle_client(connection: BudsConnection, cd: Arc<Mutex<ConnectionData>>) {
    let mut stream = connection.socket.get_stream();

    let mut buffer = [0; 2048];
    loop {
        let r = stream.read(&mut buffer[..]).await;
        if let Err(_) = r {
            return;
        }

        let num_bytes_read = r.unwrap();

        cd.lock().unwrap().msg_count.push(1);

        println!("{:?}", &buffer[0..num_bytes_read]);
    }
}
