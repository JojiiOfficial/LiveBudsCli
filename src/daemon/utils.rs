use blurz::BluetoothDevice;

/// Checks whether a device is a pair of buds live
pub fn is_bt_device_buds_live(device: &BluetoothDevice) -> bool {
    device
        .get_uuids()
        .unwrap()
        .iter()
        .any(|s| s.to_lowercase() == "00001101-0000-1000-8000-00805f9b34fb")
}

/// Converts a str to a boolean. All undefineable
/// values are false
pub fn str_to_bool<S: AsRef<str>>(s: S) -> bool {
    matches!(
        s.as_ref().to_lowercase().as_str(),
        "1" | "true" | "yes" | "y" | "enabled" | "on"
    )
}

/// return true if s can be represented as a bool
pub fn is_str_bool<S: AsRef<str>>(s: S) -> bool {
    matches!(
        s.as_ref().to_lowercase().as_str(),
        "1" | "true"
            | "yes"
            | "y"
            | "0"
            | "no"
            | "n"
            | "false"
            | "enabled"
            | "on"
            | "off"
            | "disabled"
    )
}
