use super::Request;
use super::{request_handler::get_err, Response};

use crate::daemon::{buds_config::Config, buds_info::BudsInfoInner, utils};

use async_std::sync::{Arc, Mutex};

// Set the value of a config option for a device
pub async fn set_value(payload: &Request, address: String, config: Arc<Mutex<Config>>) -> String {
    let mut config = config.lock().await;

    // Check if device already has a config entry
    // (should be available but you never know)
    if !config.has_device_config(&address) {
        return get_err("Device has no config!");
    }

    // Check required fields set
    if payload.opt_param1.is_none() || payload.opt_param2.is_none() {
        return get_err("Missing parameter");
    }

    let key = payload.opt_param1.clone().unwrap();
    let value = utils::str_to_bool(payload.opt_param2.clone().unwrap());

    // Get the right config entry mutable
    let cfg = config.get_device_config_mut(&address);
    if cfg.is_none() {
        return get_err("error getting config!");
    }
    let mut cfg = cfg.unwrap();

    // Set the right value of the config
    match key.as_str() {
        "auto_pause" => cfg.auto_pause_music = Some(value),
        "auto_play" => cfg.auto_resume_music = Some(value),
        "smart_sink" => cfg.smart_sink = Some(value),
        "low_battery_notification" => cfg.low_battery_notification = Some(value),
        _ => {
            return get_err("Invalid key");
        }
    }

    // Try to save the config
    if let Err(err) = config.save().await {
        return get_err(format!("Err saving config: {}", err).as_str());
    }

    let a: Response<BudsInfoInner> = Response::new_success(address.clone(), None);
    serde_json::to_string(&a).unwrap()
}
