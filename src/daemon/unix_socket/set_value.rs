use super::{
    super::{
        buds_info::{BudsInfo, BudsInfoInner},
        utils,
    },
    request_handler::get_err,
    Request, Response,
};

use galaxy_buds_rs::{
    message::{
        ambient_mode,
        bud_property::{BudProperty, EqualizerType, Side, TouchpadOption},
        lock_touchpad::{self, ExtLockTouchpad},
        set_noise_reduction, set_touchpad_option,
        simple::new_equalizer,
    },
    model::Feature,
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
        "noise_reduction" => set_anc(value, buds_info).await,

        // Set Touchpad lock
        "lock_touchpad" => lock_touchpad(value, buds_info, opt_param3).await,

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

        "touchpad_action" | "ambient_volume" => match value.parse::<u8>() {
            Ok(val) => match key {
                "touchpad_action" => set_touchpad_action(val, buds_info, opt_param3).await,
                "ambient_volume" => set_ambient_volume_cmd(val, buds_info).await,

                _ => Err("Invalid key to set to".to_string()),
            },
            Err(_) => Err("could not parse value".to_string()),
        },

        _ => Err("Invalid key to set to".to_string()),
    }
}

async fn lock_touchpad(
    value: &str,
    buds_info: &mut BudsInfo,
    p3: &Option<String>,
) -> Result<(), String> {
    if buds_info.has_feature(Feature::ExtTouchpadLock) {
        return lock_touchpad_ext(value, buds_info, p3).await;
    }

    let value = utils::str_to_bool(&value);
    let msg = lock_touchpad::new(value);
    buds_info.send(msg).await?;
    buds_info.inner.touchpads_blocked = value;
    Ok(())
}

async fn lock_touchpad_ext(
    value: &str,
    buds_info: &mut BudsInfo,
    p3: &Option<String>,
) -> Result<(), String> {
    let mut msg = ExtLockTouchpad::from_ext_tap_lock_status(buds_info.inner.tab_lock_status);

    // 'value' is true if lock but we want to know when to enable them here
    let val = !utils::str_to_bool(&value);

    if let Some(p3) = p3 {
        match p3.to_lowercase().as_str() {
            "tap" => msg.tap_on = val,
            "doubletap" | "dtap" => msg.double_tap = val,
            "tripletap" | "ttap" => msg.tripple_tap = val,
            "hold" => msg.touch_and_hold = val,
            _ => {
                println!("Unsupported touchpad function {p3:?}. Choose from {{tap, doubletap, tripletap, hold}}");
                return Ok(());
            }
        };
    } else {
        msg.tap_on = val;
        msg.touch_and_hold = val;
    }
    msg.touch_controls = true;

    println!("{msg:#?}");
    buds_info.send(msg).await?;

    buds_info.inner.tab_lock_status.tap_on = msg.tap_on;
    buds_info.inner.tab_lock_status.triple_tap_on = msg.tripple_tap;
    buds_info.inner.tab_lock_status.double_tap_on = msg.double_tap;
    buds_info.inner.tab_lock_status.touch_controls_on = msg.touch_controls;
    buds_info.inner.tab_lock_status.touch_an_hold_on = msg.touch_and_hold;

    Ok(())
}

/// Set the anc status
async fn set_anc(value: &str, buds_info: &mut BudsInfo) -> Result<(), String> {
    check_feature(buds_info, Feature::Anc)?;

    let value = utils::str_to_bool(&value);
    buds_info.send(set_noise_reduction::new(value)).await?;
    buds_info.inner.noise_reduction = value;
    Ok(())
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

/// Sets the extra high ambient volume value.
async fn set_extra_high_volume(enabled: bool, buds_info: &mut BudsInfo) -> Result<(), String> {
    println!("setting extra high volume {}", enabled);

    buds_info
        .send(ambient_mode::SetExtraHighVolume::new(enabled))
        .await?;

    buds_info.inner.extra_high_ambient_volume = enabled;
    Ok(())
}

/// Sets the ambient volume.
async fn set_ambient_volume(volume: u8, buds_info: &mut BudsInfo) -> Result<(), String> {
    println!("setting ambient volume to {}", volume);

    buds_info
        .send(ambient_mode::SetAmbientVolume::new(volume))
        .await?;

    buds_info.inner.ambient_sound_volume = volume;
    Ok(())
}

/// Sets the ambient mode.
async fn set_ambient_mode(enabled: bool, buds_info: &mut BudsInfo) -> Result<(), String> {
    println!("setting ambient state to {}", enabled);

    buds_info
        .send(ambient_mode::SetAmbientMode::new(enabled))
        .await?;

    buds_info.inner.ambient_sound_enabled = enabled;
    Ok(())
}

/// Set the ambient volume level
async fn set_ambient_volume_cmd(val: u8, buds_info: &mut BudsInfo) -> Result<(), String> {
    check_feature(buds_info, Feature::AmbientSound)?;

    if val > buds_info.get_max_ambientsound_volume_level() {
        return Err("Invalid volume level".to_string());
    }

    // Enable/disable extra high ambient volume if needed or not.
    if buds_info.has_feature(Feature::ExtraHighAmbientVolume) {
        if val == 4 && !buds_info.inner.extra_high_ambient_volume {
            // Enable ambient sound if not already enabled
            if !buds_info.inner.ambient_sound_enabled {
                set_ambient_mode(true, buds_info).await?;
            }

            set_extra_high_volume(true, buds_info).await?;
        } else if buds_info.inner.extra_high_ambient_volume {
            set_extra_high_volume(false, buds_info).await?;
        }
    }

    // Enable/disable the ambient mode feature
    if val == 0 && buds_info.inner.ambient_sound_enabled {
        return set_ambient_mode(false, buds_info).await; // Don't run set_ambient_volume after disabling it.
    } else if val != 0 && !buds_info.inner.ambient_sound_enabled {
        set_ambient_mode(true, buds_info).await?;
    }

    // We don't need to send volume 0 to the buds
    if val == 0 {
        return Ok(());
    }

    set_ambient_volume(val, buds_info).await
}

/// Checks a given feature and returns an error if the feature is unsupported.
fn check_feature(buds_info: &BudsInfo, feature: Feature) -> Result<(), String> {
    if !buds_info.inner.model.has_feature(feature) {
        Err("Feature not supported by your model".to_string())
    } else {
        Ok(())
    }
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
