use blurz::{BluetoothAdapter, BluetoothDevice, BluetoothSession};

// Connect or disconnect to the buds
pub async fn change_connection_status(device_addr: String, connect: bool) -> String {
    // Init bluetooth session and adapter
    let session = BluetoothSession::create_session(None);
    if session.is_err() {
        return format!("Err: {}", session.err().unwrap().to_string());
    }
    let session = session.unwrap();
    let adapter = BluetoothAdapter::init(&session);
    if adapter.is_err() {
        return format!("Err: {}", adapter.err().unwrap().to_string());
    }
    let adapter = adapter.unwrap();
    let devices = adapter.get_device_list();
    if devices.is_err() {
        return format!("Err: {}", devices.err().unwrap().to_string());
    }

    // Find device
    let device = devices
        .unwrap()
        .iter()
        .map(|i| BluetoothDevice::new(&session, i.clone()))
        .collect::<Vec<BluetoothDevice>>()
        .into_iter()
        .find(|i| i.get_address().unwrap() == device_addr);

    if device.is_none() {
        return "Err: device not found!".to_string();
    }
    let device = device.unwrap();

    // Connect or disconnect
    if let Err(err) = {
        if connect {
            if device.is_connected().unwrap_or(false) {
                return "Device is already connected".to_owned();
            }

            device.connect(8000)
        } else {
            device.disconnect()
        }
    } {
        return err.to_string();
    }

    "success".to_string()
}
