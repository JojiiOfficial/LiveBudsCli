use super::bud_connection::BudsConnection;
use async_std::io::prelude::*;
use bluetooth_serial_port_async::{BtAddr, BtProtocol, BtSocket};
use std::marker::Send;
use std::sync::mpsc::Receiver;
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
}

pub async fn run(rec: Receiver<String>) {
    let mut connections = ConnHandler::new();

    for i in rec {
        if connections.has_device(i.as_str()) {
            println!("dev already connected!");
            continue;
        }

        let connection = connect_rfcomm(&i).await;

        if let Err(err) = connection {
            eprintln!("Error connecting to rfcomm:{:?}", err);
            continue;
        }

        println!("Connected successfully to Buds live!");
        connections.add_device(i.to_owned());
        async_std::task::spawn(handle_client(connection.unwrap()));
    }
}

/// Connect to buds live via rfcomm proto
async fn connect_rfcomm<S: AsRef<str>>(addr: S) -> Result<BudsConnection, Box<dyn Error>> {
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

async fn handle_client(connection: BudsConnection) {
    let mut stream = connection.socket.get_stream();

    let mut buffer = [0; 2048];
    loop {
        let num_bytes_read = stream.read(&mut buffer[..]).await.unwrap();
        let buff = &buffer[0..num_bytes_read];
        println!("{:?}", &buff[0..num_bytes_read]);
    }
}
