use super::socket_client::{self, SocketClient};
use super::utils;
use crate::daemon::bud_connection::BudsInfoInner;
use crate::daemon::utils::{is_str_bool, str_to_bool};

use clap::ArgMatches;
use galaxy_buds_live_rs::message::bud_property::EqualizerType;

/// Set a value
pub fn set(sc: &mut SocketClient, app: &ArgMatches) {
    let value = app.value_of("value").unwrap();
    let skey = app.value_of("key").unwrap();
    let key = match parse_key(skey) {
        Some(k) => k,
        None => {
            println!("Invalid key: {}", skey);
            return;
        }
    };

    if !is_value_ok(key, value) {
        println!("invalid value: '{}' for key: '{}'", value, skey);
        return;
    }
}

/// Return true if the value is allowed for the given key
fn is_value_ok(key: Key, value: &str) -> bool {
    match key {
        Key::Anc => is_str_bool(value),
        Key::Touchpadlock => is_str_bool(value),
        Key::Equalizer => parse_equalizer(value) != EqualizerType::Undetected,
    }
}

#[derive(Debug, Copy, Clone)]
enum Key {
    Anc,
    Equalizer,
    Touchpadlock,
}

fn parse_key(key: &str) -> Option<Key> {
    Some(match key.to_string().to_lowercase().as_str() {
        "nc" | "anc" | "noise_reduction" | "noise-reduction" | "noise-cancellation" => Key::Anc,
        "eq" | "equalizer" | "equalizer-type" | "equalizertype" => Key::Equalizer,
        "touchpadlock" | "tpl" | "locktouchpad" => Key::Touchpadlock,
        _ => return None,
    })
}

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
