/*
 * Handles incoming bluetooth connections and
 * forwards connection events to the connector
 */

use crate::model::Model;
use async_std::task;
use bluetooth_serial_port_async::BtSocket;
use blurz::{
    BluetoothAdapter, BluetoothDevice,
    BluetoothEvent::{self, Connected},
    BluetoothSession,
};

use std::sync::mpsc::Sender;
use std::time::Duration;

use super::rfcomm_connector::ConnectionEventData;

/// An active connection to a pair of buds
#[derive(Debug)]
pub struct BudsConnection {
    pub addr: String,
    pub socket: BtSocket,
    pub fd: i32,
}

/// Listens for new Bluethooth connections
pub async fn run(sender: Sender<ConnectionEventData>) {
    let session = &BluetoothSession::create_session(None).unwrap();
    let mut printed_adapter_missing = false;

    'outer: loop {
        let adapter = BluetoothAdapter::init(session);
        if let Err(err) = adapter {
            // Thanks blurz for implementing usable error types
            let s = err.to_string();

            // On adapter not found, wait and try to connect to it again
            // maybe the user inserted the adapter later on
            if s.contains("Bluetooth adapter not found") {
                task::sleep(Duration::from_secs(2)).await;

                if !printed_adapter_missing {
                    eprintln!("Bluetooth adapter missing!");
                }
                printed_adapter_missing = true;

                continue;
            } else {
                // Every other error should be treated as fatal error
                println!("Bluetooth error: {}", err);
                std::process::exit(1);
            }
        } else {
            printed_adapter_missing = false;
        }

        let adapter = adapter.unwrap();

        // We need this behaivor twice
        let check_device = |device: String| {
            let device = BluetoothDevice::new(&session, device);

            if is_supported_pair_of_buds(&device) {
                sender
                    .send(ConnectionEventData {
                        address: device.get_address().unwrap(),
                        model: Model::from(device.get_name().unwrap().as_str()),
                    })
                    .unwrap();
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

                check_device(device.get_id());
            }
        }

        // Handle all future connection events
        loop {
            if adapter.is_powered().is_err() {
                continue 'outer;
            }

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
}

/// Checks whether a device is a pair of buds live
pub fn is_supported_pair_of_buds(device: &BluetoothDevice) -> bool {
    device
        .get_uuids()
        .unwrap()
        .iter()
        .any(|s| s.to_lowercase() == "00001101-0000-1000-8000-00805f9b34fb")
}
