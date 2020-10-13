/*
 * Handles incoming bluetooth connections and connects
 * to the galaxy buds if available
 */

use super::utils;

use blurz::{
    BluetoothDevice,
    BluetoothEvent::{self, Connected},
    BluetoothSession,
};

use std::sync::mpsc::{self, Sender};

/// Run the bluetooth futures
pub async fn run(sender: Sender<String>) {
    let (connect_tx, connect_rx) = mpsc::channel::<String>();

    //async_std::task::spawn(run_connection_listener(connect_rx, sender));
    run_bt_listener(sender).await;
}

/// Listens for new Bluethooth connections
async fn run_bt_listener(sender: Sender<String>) {
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
                    //tx.send(device.get_address().unwrap()).unwrap();
                    sender.send(device.get_address().unwrap()).unwrap();
                } else {
                    println!("Buds disconnected");
                }
            }
        }
    }
}
