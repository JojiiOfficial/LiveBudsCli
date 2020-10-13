/*
 * Handles incoming bluetooth connections and connects
 * to the galaxy buds if available
 */

use super::bud_connection::{BudsConnection, ConnectionEventInfo};
use super::utils;

use std::sync::mpsc::Sender;
use std::{error::Error, str::FromStr};

use bluetooth_serial_port_async::{BtAddr, BtProtocol, BtSocket};
use blurz::{
    BluetoothAdapter, BluetoothDevice,
    BluetoothEvent::{self, Connected},
    BluetoothSession,
};

/// Listens for new Bluethooth connections
pub async fn run(sender: Sender<ConnectionEventInfo>) {
    let session = &BluetoothSession::create_session(None).unwrap();
    let adapter: BluetoothAdapter = BluetoothAdapter::init(session).unwrap();

    // We need this behaivor twice
    let check_device = |device: String, connected: bool| {
        let device = BluetoothDevice::new(&session, device);
        if utils::is_bt_device_buds_live(&device) {
            sender
                .send(ConnectionEventInfo::new(
                    device.get_address().unwrap(),
                    connected,
                ))
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

/// Connect to buds live via rfcomm proto
pub fn connect_rfcomm<S: AsRef<str>>(addr: S) -> Result<BudsConnection, Box<dyn Error>> {
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
