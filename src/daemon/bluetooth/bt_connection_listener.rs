/*
 * Handles incoming bluetooth connections and
 * forwards connection events to the connector
 */

use super::super::utils;

use bluetooth_serial_port_async::BtSocket;
use blurz::{
    BluetoothAdapter, BluetoothDevice,
    BluetoothEvent::{self, Connected},
    BluetoothSession,
};

use std::sync::mpsc::Sender;

/// An active connection to a pair of buds
#[derive(Debug)]
pub struct BudsConnection {
    pub addr: String,
    pub socket: BtSocket,
    pub fd: i32,
}

/// Listens for new Bluethooth connections
pub async fn run(sender: Sender<String>) {
    let session = &BluetoothSession::create_session(None).unwrap();
    let adapter: BluetoothAdapter = BluetoothAdapter::init(session).unwrap();

    // We need this behaivor twice
    let check_device = |device: String| {
        let device = BluetoothDevice::new(&session, device);
        if utils::is_bt_device_buds_live(&device) {
            sender.send(device.get_address().unwrap()).unwrap();
        }
    };

    // check if a pair of buds is already connected!
    if let Ok(devices) = adapter.get_device_list() {
        for device in devices {
            let device = BluetoothDevice::new(&session, device);
            let is_connected = device.is_connected();
            if is_connected.is_err() || !is_connected.unwrap() {
                continue;
            }

            println!("check device {:?}", device);
            check_device(device.get_id());
        }
    }

    // Handle all future connection events
    loop {
        for event in session.incoming(1000).map(BluetoothEvent::from) {
            if event.is_none() {
                continue;
            }

            if let Connected {
                object_path,
                connected,
            } = event.unwrap()
            {
                if !connected {
                    continue;
                }

                check_device(object_path);
            }
        }
    }
}
