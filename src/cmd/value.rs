use super::socket_client::{self, SocketClient};
use super::utils;
use crate::daemon::utils::{is_str_bool, str_to_bool};

use clap::ArgMatches;
use galaxy_buds_live_rs::message::bud_property::{BudProperty, EqualizerType};

/// Set a value
pub fn set(sc: &mut SocketClient, app: &ArgMatches) {
    let value = app.value_of("value").unwrap();
    let skey = app.value_of("key").unwrap();
    let key = match Key::parse(skey) {
        Some(k) => k,
        None => {
            println!("Invalid key: {}", skey);
            return;
        }
    };

    // Check value input
    if !is_value_ok(key, value) {
        println!("invalid value: '{}' for key: '{}'", value, skey);
        return;
    }

    // Build request payload
    let request = socket_client::new_set_value_request(
        utils::get_device_from_app(&app),
        key.value(),
        get_value(key, value),
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
fn get_value(key: Key, value: &str) -> String {
    match key {
        Key::Anc | Key::Touchpadlock => str_to_bool(value).to_string(),
        Key::Touchpad => (!str_to_bool(value)).to_string(),
        Key::Equalizer => parse_equalizer(value).encode().to_string(),
    }
}

/// Return true if the value is allowed for the given key
fn is_value_ok(key: Key, value: &str) -> bool {
    match key {
        Key::Touchpadlock | Key::Touchpad | Key::Anc => is_str_bool(value),
        Key::Equalizer => parse_equalizer(value) != EqualizerType::Undetected,
    }
}

// parse equalizer strings to enum variants
fn parse_equalizer(value: &str) -> EqualizerType {
    match value.to_lowercase().as_str() {
        "normal" | "off" => EqualizerType::Normal,
        "bass" | "bb" => EqualizerType::BassBoost,
        "soft" => EqualizerType::Soft,
        "dynamic" | "dyn" => EqualizerType::Dynamic,
        "clear" => EqualizerType::Clear,
        "treble" => EqualizerType::TrebleBoost,
        _ => EqualizerType::Undetected,
    }
}

#[derive(Debug, Copy, Clone)]
enum Key {
    Anc,
    Equalizer,
    Touchpadlock,
    Touchpad, // I prefer to type 'set touchpad 1' instead of 'set touchpadlock 0'
}

impl Key {
    fn value(&self) -> String {
        String::from(match *self {
            Key::Anc => "noise_reduction",
            Key::Equalizer => "equalizer",
            Key::Touchpadlock => "lock_touchpad",
            Key::Touchpad => "lock_touchpad",
        })
    }

    fn parse(key: &str) -> Option<Key> {
        Some(match key.to_string().to_lowercase().as_str() {
            "nc" | "anc" | "noise_reduction" | "noise-reduction" => Key::Anc,
            "eq" | "equalizer" | "equalizer-type" | "equalizertype" => Key::Equalizer,
            "touchpadlock" | "tpl" | "locktouchpad" => Key::Touchpadlock,
            "touchpad" => Key::Touchpad,
            _ => return None,
        })
    }
}
