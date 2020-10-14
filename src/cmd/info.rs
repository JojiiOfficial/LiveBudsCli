use super::socket_client::{self, SocketClient};
use crate::daemon::unix_request_handler::{Request, Response};
use clap::ArgMatches;

pub fn show(sc: &mut SocketClient, app: ArgMatches) {
    let status = sc.do_request(socket_client::new_status_request(None));
    println!("{:#?}", status);
}
