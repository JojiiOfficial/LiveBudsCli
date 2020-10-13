use blurz::BluetoothDevice;

/// Checks whether a device is a pair of buds live
pub fn is_bt_device_buds_live(device: &BluetoothDevice) -> bool {
    device
        .get_uuids()
        .unwrap()
        .into_iter()
        .find(|s| s.to_lowercase() == "00001101-0000-1000-8000-00805f9b34fb")
        .is_some()
}
