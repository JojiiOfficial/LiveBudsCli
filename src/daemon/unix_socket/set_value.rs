use super::request_handler::get_err;
use super::{Request, Response};

use super::super::buds_info::{BudsInfo, BudsInfoInner};
use super::super::utils;

use galaxy_buds_rs::{
    message::{
        bud_property::{BudProperty, EqualizerType, Side, TouchpadOption},
        lock_touchpad, set_noise_reduction, set_touchpad_option,
        simple::new_equalizer,
    },
    model::Model,
};

// Parses the payload and runs the actual set-option request
pub async fn set(payload: &Request, device_data: &mut BudsInfo) -> String {
    // Check required fields set
    if payload.opt_param1.is_none() || payload.opt_param2.is_none() {
        return get_err("Missing parameter");
    }

    let key = payload.opt_param1.clone().unwrap();
    let value = payload.opt_param2.clone().unwrap();

    // Run desired command
    let res = set_buds_option(
        key.as_str(),
        value.as_str(),
        device_data,
        &payload.opt_param3,
    )
    .await;

    // Return success or error based on the success of the set command
    if res.is_ok() {
        let a: Response<BudsInfoInner> =
            Response::new_success(device_data.inner.address.clone(), None);
        serde_json::to_string(&a).unwrap()
    } else {
        get_err(res.err().unwrap().as_str())
    }
}

// Set the actual value
async fn set_buds_option(
    key: &str,
    value: &str,
    buds_info: &mut BudsInfo,
    opt_param3: &Option<String>,
) -> Result<(), String> {
    match key {
        // Set noise reduction
        "noise_reduction" => {
            let value = utils::str_to_bool(&value);
            let msg = set_noise_reduction::new(value);
            let res = buds_info.send(msg).await;
            if res.is_ok() {
                buds_info.inner.noise_reduction = value;
            }
            res
        }

        // Set Touchpad lock
        "lock_touchpad" => {
            let value = utils::str_to_bool(&value);
            let msg = lock_touchpad::new(value);
            let res = buds_info.send(msg).await;
            if res.is_ok() {
                buds_info.inner.touchpads_blocked = value;
            }
            res
        }

        // Set EqualizerType command
        "equalizer" => match value.parse::<u8>() {
            Ok(val) => {
                let eq_type = EqualizerType::decode(val);
                let res = buds_info.send(new_equalizer(eq_type)).await;
                if res.is_ok() {
                    buds_info.inner.equalizer_type = eq_type;
                }
                res
            }
            Err(_) => Err("could not parse value".to_string()),
        },

        // Touchpad option
        "touchpad_action" => match value.parse::<u8>() {
            Ok(val) => set_touchpad_action(val, buds_info, opt_param3).await,
            Err(_) => Err("could not parse value".to_string()),
        },

        // Ambient volume
        "ambient_volume" => match value.parse::<u8>() {
            Ok(val) => set_ambient_volume(val, buds_info).await,
            Err(_) => Err("could not parse value".to_string()),
        },
        _ => Err("Invalid key to set to".to_string()),
    }
}

/// Set the touchpad action
async fn set_touchpad_action(
    val: u8,
    buds_info: &mut BudsInfo,
    opt_param3: &Option<String>,
) -> Result<(), String> {
    let option = TouchpadOption::decode(val);
    let mut left = buds_info.inner.touchpad_option_left;
    let mut right = buds_info.inner.touchpad_option_right;

    if opt_param3.is_some() {
        let side = utils::str_to_side(opt_param3.as_ref().unwrap());
        if side.is_none() {
            return Err("Invalid side".to_string());
        }
        match side.unwrap() {
            Side::Left => left = option,
            Side::Right => right = option,
        }
    } else {
        left = option;
        right = option;
    }

    let msg = set_touchpad_option::new(left, right);
    let res = buds_info.send(msg).await;
    if res.is_ok() {
        buds_info.inner.touchpad_option_left = left;
        buds_info.inner.touchpad_option_right = right;
    }
    res
}

/// Set the ambient volume level
async fn set_ambient_volume(val: u8, buds_info: &mut BudsInfo) -> Result<(), String> {
    if buds_info.inner.model != Model::BudsPlus {
        //     return Err("Not supported for your model".to_string());
    }

    if val > 4 {
        return Err("Invalid volume level".to_string());
    }

    if buds_info.inner.ambient_sound_volume == val {
        return Ok(());
    }

    if val == 4 && !buds_info.inner.extra_high_ambient_volume {
        println!("enable extra high");

        buds_info.inner.extra_high_ambient_volume = true;
    } else if buds_info.inner.extra_high_ambient_volume {
        println!("disable extra high");

        buds_info.inner.extra_high_ambient_volume = false;
    }

    if val == 0 {
        println!("disable ambient");
    } else if buds_info.inner.ambient_sound_volume == 0 {
        println!("enableble ambient");
    }

    println!("level: {}", val);
    buds_info.inner.ambient_sound_volume = val;
    Ok(())
}

// Toggle a given value
pub async fn toggle(payload: &Request, device_data: &mut BudsInfo) -> String {
    // Check required fields set
    if payload.opt_param1.is_none() {
        return get_err("Missing parameter");
    }

    let key = payload.opt_param1.clone().unwrap();
    let value = {
        match key.as_str() {
            "noise_reduction" => (!device_data.inner.noise_reduction).to_string(),
            "lock_touchpad" => (!device_data.inner.touchpads_blocked).to_string(),
            _ => {
                return get_err("Invalid key");
            }
        }
    };

    // Run desired command
    let res = set_buds_option(
        key.as_str(),
        value.as_str(),
        device_data,
        &payload.opt_param3,
    )
    .await;

    // Return success or error based on the success of the set command
    if res.is_ok() {
        let a: Response<BudsInfoInner> =
            Response::new_success(device_data.inner.address.clone(), None);
        serde_json::to_string(&a).unwrap()
    } else {
        get_err(res.err().unwrap().as_str())
    }
}
