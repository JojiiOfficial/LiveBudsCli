use super::{
    socket_client::{self, SocketClient},
    utils,
};
use clap::ArgMatches;

pub fn connect(sc: &mut SocketClient, app: &ArgMatches) {
    let response = sc
        .do_request(socket_client::new_connect_request(
            utils::get_device_from_app(&app),
        ))
        .unwrap();

    println!("{}", response);
}

pub fn disconnect(sc: &mut SocketClient, app: &ArgMatches) {
    let response = sc
        .do_request(socket_client::new_disconnect_request(
            utils::get_device_from_app(&app),
        ))
        .unwrap();

    println!("{}", response);
}
