/*
 * Handles incoming bluetooth connections and
 * forwards connection events to the connector
 */

use super::super::utils;

use async_std::task;
use bluetooth_serial_port_async::BtSocket;
use blurz::{
    BluetoothAdapter, BluetoothDevice,
    BluetoothEvent::{self, Connected},
    BluetoothSession,
};

use std::sync::mpsc::Sender;
use std::time::Duration;

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
