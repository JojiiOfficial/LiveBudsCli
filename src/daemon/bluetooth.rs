/*
 * Handles incoming bluetooth connections and connects
 * to the galaxy buds if available
 */

use super::utils;
use async_std::os::unix::net::UnixStream;
use bluetooth_serial_port_async::{BtAddr, BtProtocol, BtSocket};
use blurz::{
    BluetoothDevice,
    BluetoothEvent::{self, Connected},
    BluetoothSession,
};

use std::sync::mpsc::{self, Receiver, Sender};
use std::{error::Error, str::FromStr};

/// An active connection to a pair of buds
#[derive(Debug)]
pub struct BudsConnection {
    pub addr: String,
    pub stream: UnixStream,
}

/// Run the bluetooth futures
pub async fn run() {
    let (connect_tx, connect_rx) = mpsc::channel::<String>();

    async_std::task::spawn(run_connection_listener(connect_rx));
    run_bt_listener(connect_tx).await;
}

/// Listens for new Bluethooth connections
async fn run_bt_listener(tx: Sender<String>) {
    let session = &BluetoothSession::create_session(None).unwrap();

    loop {
        // Handle all connection events
        for event in session.incoming(10000).map(BluetoothEvent::from) {
            if event.is_none() {
                continue;
            }
            let event = event.unwrap();

            if let Connected {
                object_path,
                connected,
            } = event
            {
                let device = BluetoothDevice::new(&session, object_path.clone());

                if !utils::is_bt_device_buds_live(&device) {
                    println!("Non buds connected!");
                    continue;
                }

                if connected {
                    println!("Buds connected!!!");
                    tx.send(device.get_address().unwrap()).unwrap();
                } else {
                    println!("Buds disconnected");
                }
            }
        }
    }
}

/// Connects to buds if available
async fn run_connection_listener(rx: Receiver<String>) {
    for i in rx {
        println!("{:?}", i);
        let connection = connect_rfcomm(i).await;
        if let Err(err) = connection {
            eprintln!("Cant get rfcomm channel to work: {}", err);
            continue;
        }

        let connection = connection.unwrap();
        println!("{:?}", connection);
    }
}

/// Connect to buds live via rfcomm proto
async fn connect_rfcomm<S: AsRef<str>>(addr: S) -> Result<BudsConnection, Box<dyn Error>> {
    let mut socket = BtSocket::new(BtProtocol::RFCOMM)?;
    let address = BtAddr::from_str(addr.as_ref()).unwrap();

    socket.connect(&address)?;
    let stream = socket.get_stream();

    Ok(BudsConnection {
        addr: addr.as_ref().to_owned(),
        stream,
    })
}
