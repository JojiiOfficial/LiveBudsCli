/*
 * Handles incoming bluetooth connections and connects
 * to the galaxy buds if available
 */

use super::bud_connection::ConnectInfo;
use super::utils;

use blurz::{
    BluetoothAdapter, BluetoothDevice,
    BluetoothEvent::{self, Connected},
    BluetoothSession,
};

use std::sync::mpsc::Sender;

/// Listens for new Bluethooth connections
pub async fn run(sender: Sender<ConnectInfo>) {
    let session = &BluetoothSession::create_session(None).unwrap();
    let adapter: BluetoothAdapter = BluetoothAdapter::init(session).unwrap();

    // We need this behaivor twice
    let check_device = |device: String, connected: bool| {
        let device = BluetoothDevice::new(&session, device);
        if utils::is_bt_device_buds_live(&device) {
            sender
                .send(ConnectInfo::new(device.get_address().unwrap(), connected))
                .unwrap();
        }
    };

    // check if a pair of buds is already connected!
    if let Ok(devices) = adapter.get_device_list() {
        for device in devices {
            check_device(device, true);
        }
    }

    // Handle all future connection events
    loop {
        for event in session.incoming(10000).map(BluetoothEvent::from) {
            if event.is_none() {
                continue;
            }

            if let Connected {
                object_path,
                connected,
            } = event.unwrap()
            {
                check_device(object_path, connected);
            }
        }
    }
}
