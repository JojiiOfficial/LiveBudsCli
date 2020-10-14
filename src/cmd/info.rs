use super::socket_client::{self, SocketClient};
use super::utils;
use crate::daemon::buds_info::BudsInfoInner;

use clap::ArgMatches;

/// show status of given address
pub fn show(sc: &mut SocketClient, app: &ArgMatches) {
    // Do request
    let status = sc
        .do_request(socket_client::new_status_request(
            utils::get_device_from_app(&app),
        ))
        .unwrap();

    // Print as json if user desires so
    if utils::print_as_json(&app) {
        println!("{}", status);
        return;
    }

    // Convert to info response
    let status = socket_client::to_buds_info(status);
    let res: BudsInfoInner = utils::unwrap_response(&status).unwrap();

    // TODO make more pretty than pretty debug
    println!("{:#?}", res);
}
