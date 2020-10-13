mod daemon;

use async_std;
use clap::{clap_app, crate_version};

#[async_std::main]
async fn main() {
    let clap = clap_app!(livebuds => (version:crate_version!())
                         (author:"Jojii S")
                         (about:"Control your Galaxy Buds live from cli")
                         (@arg daemon: -d --daemon "Starts the daemon"))
    .get_matches();

    // run only the daemon if desired
    if clap.is_present("daemon") {
        daemon::run_daemon().await;
        println!("daemon exit");

        return;
    }

    println!("other commands");
}
