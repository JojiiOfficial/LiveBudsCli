/*
 * Handles incoming bluetooth connections and
 * forwards connection events to the connector
 */

use bluetooth_serial_port_async::BtSocket;
use blurz::{
    BluetoothAdapter, BluetoothDevice,
    BluetoothEvent::{self, Connected},
    BluetoothSession,
};
use galaxy_buds_rs::model::Model;
use log::debug;

use std::sync::mpsc::Sender;
use std::time::Duration;

use super::rfcomm_connector::ConnectionEventData;

/// An active connection to a pair of buds
#[derive(Debug)]
pub struct BudsConnection {
    pub addr: String,
    pub socket: BtSocket,
    // pub fd: i32,
}

/// Listens for new Bluethooth connections
pub fn run(sender: Sender<ConnectionEventData>) {
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
                std::thread::sleep(Duration::from_secs(2));

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

        // check if a pair of buds is already connected!
        if let Ok(devices) = adapter.get_device_list() {
            for device in devices {
                let device = BluetoothDevice::new(&session, device);
                let is_connected = device.is_connected();
                if is_connected.is_err() || !is_connected.unwrap() {
                    continue;
                }

                check_device(&sender, &session, device.get_id());
            }
        }

        // Handle all future connection events
        loop {
            if adapter.is_powered().is_err() {
                continue 'outer;
            }

            for event in session.incoming(10000000).map(BluetoothEvent::from) {
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

                    println!("device {:#?}", object_path);
                    check_device(&sender, &session, object_path);
                }
            }
        }
    }
}

// We need this behaivor twice
fn check_device(sender: &Sender<ConnectionEventData>, session: &BluetoothSession, device: String) {
    let device = BluetoothDevice::new(session, device);

    if !supported_device(&device) {
        let name = device.get_name().unwrap();
        debug!("Not supported: {name}");
        return;
    }

    sender
        .send(ConnectionEventData {
            address: device.get_address().unwrap(),
            model: name_to_model(device.get_name().unwrap().as_str()),
        })
        .unwrap();
}

/// Checks whether a device is a pair of buds live
pub fn supported_device(device: &BluetoothDevice) -> bool {
    device
        .get_uuids()
        .unwrap()
        .iter()
        .any(|s| s.to_lowercase() == "00001101-0000-1000-8000-00805f9b34fb")
}

/// Gives devices model from its name
fn name_to_model(device_name: &str) -> Model {
    let device_name = device_name.to_lowercase();

    if device_name.contains("buds live") {
        Model::BudsLive
    } else if device_name.contains("buds pro") {
        Model::BudsPro
    } else if device_name.contains("buds 2 pro") {
        Model::BudsPro2
    } else if device_name.contains("buds+") {
        Model::BudsPlus
    } else if device_name.contains("buds2") {
        Model::Buds2
    } else {
        Model::Buds
    }
}
