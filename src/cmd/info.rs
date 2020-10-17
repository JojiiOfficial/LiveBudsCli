use super::socket_client::{self, SocketClient};
use super::utils;
use crate::daemon::buds_info::BudsInfoInner;

use blurz::{BluetoothAdapter, BluetoothDevice, BluetoothSession};
use clap::ArgMatches;
use galaxy_buds_live_rs::message::bud_property::Placement;

/// show status of given address
pub fn show(sc: &mut SocketClient, app: &ArgMatches) {
    // Do request
    let status = sc
        .do_request(socket_client::new_status_request(
            utils::get_device_from_app(&app),
        ))
        .unwrap();

    // Print as json if user desires so
    if utils::print_as_json(&app) {
        println!("{}", status);
        return;
    }

    // Convert to info response
    let status = socket_client::to_buds_info(status);
    let res: BudsInfoInner = utils::unwrap_response(&status).unwrap();

    let bt_name = get_bt_device_name(&res.address).unwrap_or_else(|| res.address.clone());

    println!("Info for '{}':", bt_name);
    println!();
    println!("Battery:\tL: {}%, R: {}%", res.batt_left, res.batt_right);

    // If one bean is not in the case, its batterystatus
    // can't be deterimned and the buds will always return 100%
    if res.placement_left == Placement::InOpenCase
        || res.placement_left == Placement::InCloseCase
        || res.placement_right == Placement::InCloseCase
        || res.placement_right == Placement::InOpenCase
    {
        println!("Case:\t\t{}%", res.batt_case);
    }

    println!("Equalizer:\t{:?}", res.equalizer_type);
    println!("ANC:\t\t{}", {
        if res.noise_reduction {
            "Enabled"
        } else {
            "Disabled"
        }
    });
    println!("Touchpads:\t{}", {
        if res.touchpads_blocked {
            "Blocked"
        } else {
            "Enabled"
        }
    });
    println!("Left option:\t{:?}", res.touchpad_option_left);
    println!("Right option:\t{:?}", res.touchpad_option_right);
}

fn get_bt_device_name<S: AsRef<str>>(dev_addr: S) -> Option<String> {
    let session = BluetoothSession::create_session(None).ok()?;
    let adapter = BluetoothAdapter::init(&session).ok()?;
    let devices = adapter.get_device_list().ok()?;

    for i in devices.into_iter() {
        let dev = BluetoothDevice::new(&session, i);
        if !dev.is_connected().ok()? {
            continue;
        }

        if dev.get_address().ok()? != dev_addr.as_ref() {
            continue;
        }

        return Some(dev.get_name().ok()?);
    }

    None
}
