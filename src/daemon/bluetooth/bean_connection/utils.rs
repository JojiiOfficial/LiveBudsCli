use galaxy_buds_live_rs::message::bud_property::Placement;
use mpris::{Player, PlayerFinder};
use notify_rust::Notification;

fn get_finder() -> Option<PlayerFinder> {
    PlayerFinder::new().ok()
}

fn get_player<'a>(finder: &'a PlayerFinder) -> Option<Player> {
    finder.find_active().ok()
}

pub fn try_pause() -> bool {
    get_finder()
        .and_then(|finder| get_player(&finder).and_then(|player| player.pause().ok()))
        .is_some()
}

pub fn try_play() -> bool {
    get_finder()
        .and_then(|finder| get_player(&finder).and_then(|player| player.play().ok()))
        .is_some()
}

pub fn is_placed_state(left: Placement, right: Placement) -> bool {
    left == Placement::InOpenCase && right == Placement::InOpenCase
}

pub fn is_some_wearing_state(left: Placement, right: Placement) -> bool {
    left == Placement::Ear || right == Placement::Ear
}

pub fn is_wearing_state(left: Placement, right: Placement) -> bool {
    left == Placement::Ear && right == Placement::Ear
}

pub fn get_desktop_notification(l_batt: i8, r_batt: i8) -> Notification {
    Notification::new()
        .summary("Buds Live battery low")
        .body(
            format!(
                "The battery of your Galaxy buds live is pretty low: (L: {}%, R: {}%)",
                l_batt, r_batt
            )
            .as_str(),
        )
        .icon("battery")
        .to_owned()
}
