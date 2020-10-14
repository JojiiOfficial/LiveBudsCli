use super::socket_client::{self, SocketClient};
use super::utils::get_device_from_app;
use crate::daemon::bud_connection::BudsInfoInner;

use clap::ArgMatches;

use std::process::exit;

pub fn show(sc: &mut SocketClient, app: ArgMatches) {
    let device = get_device_from_app(&app);

    let status = sc
        .do_request(socket_client::new_status_request(device))
        .unwrap();
    if status.is_success() {
        let payload: BudsInfoInner = status.payload.unwrap();
        println!("{:#?}", payload);
    } else {
        println!("{}", status.status_message.unwrap());
        exit(1);
    }
}
