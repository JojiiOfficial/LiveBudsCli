use super::socket_client::{self, SocketClient};
use super::utils;
use crate::daemon::bud_connection::BudsInfoInner;

use clap::ArgMatches;

/// show status of given address
pub fn show(sc: &mut SocketClient, app: &ArgMatches) {
    let status = sc
        .do_request(socket_client::new_status_request(
            utils::get_device_from_app(&app),
        ))
        .unwrap();

    if utils::print_as_json(&app) {
        println!("{}", status);
        return;
    }

    let status = socket_client::to_buds_info(status);

    let res: BudsInfoInner = utils::unwrap_response(&status).unwrap();
    println!("{:#?}", res);
}
