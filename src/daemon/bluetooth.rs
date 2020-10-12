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
use std::sync::mpsc::{self, Receiver, Sender};

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

                println!("Buds connected!!!");
                tx.send("lmao".to_owned()).unwrap();
            }
        }
    }
}

/// Connects to buds if available
async fn run_connection_listener(rx: Receiver<String>) {
    for i in rx {
        println!("{:?}", i);
    }
}
