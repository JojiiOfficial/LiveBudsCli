/*
 * Handles incoming bluetooth connections and connects
 * to the galaxy buds if available
 */

use super::bud_connection::ConnectInfo;
use super::utils;

use blurz::{
    BluetoothDevice,
    BluetoothEvent::{self, Connected},
    BluetoothSession,
};

use std::sync::mpsc::Sender;

/// Listens for new Bluethooth connections
pub async fn run(sender: Sender<ConnectInfo>) {
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

                sender
                    .send(ConnectInfo::new(device.get_address().unwrap(), connected))
                    .unwrap();
            }
        }
    }
}
