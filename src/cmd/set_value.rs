use super::socket_client::{self, SocketClient};
use super::utils;
use crate::daemon::utils::{is_str_bool, str_to_bool};

use clap::ArgMatches;
use galaxy_buds_rs::message::bud_property::{BudProperty, EqualizerType, TouchpadOption};

/// Set a value
pub fn set(sc: &mut SocketClient, app: &ArgMatches, toggle: bool, value: &str) {
    let skey = app.value_of("key").unwrap();
    let key = match Key::parse(skey) {
        Some(k) => k,
        None => {
            println!("Invalid key: {}", skey);
            return;
        }
    };

    // Check value input
    if !toggle && !is_value_ok(key, value) {
        println!("invalid value: '{}' for key: '{}'", value, skey);
        return;
    }

    // Build request payload
    let mut request = socket_client::new_set_value_request(
        utils::get_device_from_app(&app),
        key.value(),
        get_value(key, value),
        toggle,
    );

    // Set opt param if present
    if app.is_present("opt") {
        request.opt_param3 = app.value_of("opt").map(|s| s.to_owned());
    }

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
    } else if let Some(err_msg) = res.status_message {
        println!("Error: {}", err_msg);
    } else {
        println!("Error!")
    }
}

/// Return the actual value required for the payload
fn get_value(key: Key, value: &str) -> String {
    match key {
        Key::Anc | Key::Touchpadlock => str_to_bool(value).to_string(),
        Key::Touchpad => (!str_to_bool(value)).to_string(),
        Key::Equalizer => parse_equalizer(value).encode().to_string(),
        Key::TapAction => parse_tap_action(value).encode().to_string(),
        Key::AmbientSound => value.to_string(),
    }
}

/// Return true if the value is allowed for the given key
fn is_value_ok(key: Key, value: &str) -> bool {
    match key {
        Key::Touchpadlock | Key::Touchpad | Key::Anc => is_str_bool(value),
        Key::Equalizer => parse_equalizer(value) != EqualizerType::Undetected,
        Key::TapAction => parse_tap_action(value) != TouchpadOption::Undetected,
        Key::AmbientSound => utils::is_number(value),
    }
}

// parse equalizer strings to enum variants
fn parse_tap_action(value: &str) -> TouchpadOption {
    match value.to_lowercase().as_str() {
        "volume" => TouchpadOption::Volume,
        "spotify" => TouchpadOption::Spotify,
        "voice-command" => TouchpadOption::VoiceCommand,
        "anc" => TouchpadOption::NoiseCanceling,
        "disconnect" => TouchpadOption::Disconnect,
        _ => TouchpadOption::Undetected,
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
    Touchpad, // I prefer 'set touchpad 1' over 'set touchpadlock 0'
    TapAction,
    AmbientSound,
}

impl Key {
    fn value(&self) -> String {
        String::from(match *self {
            Key::Anc => "noise_reduction",
            Key::Equalizer => "equalizer",
            Key::Touchpadlock => "lock_touchpad",
            Key::Touchpad => "lock_touchpad",
            Key::TapAction => "touchpad_action",
            Key::AmbientSound => "ambient_volume",
        })
    }

    fn parse(key: &str) -> Option<Key> {
        Some(match key.to_string().to_lowercase().as_str() {
            "anc" => Key::Anc,
            "equalizer" => Key::Equalizer,
            "touchpadlock" => Key::Touchpadlock,
            "touchpad" => Key::Touchpad,
            "tap-action" => Key::TapAction,
            "ambientsound" => Key::AmbientSound,
            _ => return None,
        })
    }
}
