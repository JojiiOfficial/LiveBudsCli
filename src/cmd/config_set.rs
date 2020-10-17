use super::socket_client::{self, SocketClient};
use super::utils;
use crate::daemon::utils::{is_str_bool, str_to_bool};

use clap::ArgMatches;

/// Set a value
pub fn set(sc: &mut SocketClient, app: &ArgMatches) {
    let value = app.value_of("value").unwrap_or_default();
    let skey = app.value_of("key").unwrap();
    let key = match Key::parse(skey) {
        Some(k) => k,
        None => {
            println!("Invalid key: {}", skey);
            return;
        }
    };

    // Check value input
    if !is_value_ok(value) {
        println!("invalid value: '{}' for key: '{}'", value, skey);
        return;
    }

    // Build request payload
    let request = socket_client::new_set_config_request(
        utils::get_device_from_app(&app),
        key.value(),
        get_value(value),
    );

    // Do unix_socket request
    let res = match sc.do_request(request) {
        Ok(k) => k,
        Err(err) => {
            eprintln!("{:?}", err);
            return;
        }
    };

    // print as json if user desires so
    if utils::print_as_json(&app) {
        println!("{}", res);
        return;
    }

    // Print response in a human readable way
    let res = socket_client::to_response::<String>(&res);
    if res.is_success() {
        println!("Success");
    } else {
        if let Some(err_msg) = res.status_message {
            println!("Error: {}", err_msg);
        } else {
            println!("Error!")
        }
    }
}

/// Return true if the value is allowed for the given key
fn get_value(value: &str) -> String {
    str_to_bool(value).to_string()
}

/// Return true if the value is allowed for the given key
fn is_value_ok(value: &str) -> bool {
    is_str_bool(value)
}

#[derive(Debug, Copy, Clone)]
enum Key {
    AutoPause,
    AutoPlay,
    LowBatteryNotification,
}

impl Key {
    fn value(&self) -> String {
        String::from(match *self {
            Key::AutoPause => "auto_pause",
            Key::AutoPlay => "auto_play",
            Key::LowBatteryNotification => "low_battery_notification",
        })
    }

    fn parse(key: &str) -> Option<Key> {
        Some(match key.to_string().to_lowercase().as_str() {
            "auto-pause" => Key::AutoPause,
            "auto-play" => Key::AutoPlay,
            "low-battery-notification" => Key::LowBatteryNotification,
            _ => return None,
        })
    }
}
