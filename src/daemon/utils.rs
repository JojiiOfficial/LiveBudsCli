use galaxy_buds_live_rs::message::bud_property::Side;

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

pub fn str_to_side<S: AsRef<str>>(s: S) -> Option<Side> {
    Some(match s.as_ref() {
        "left" | "l" => Side::Left,
        "right" | "r" => Side::Right,
        _ => return None,
    })
}
