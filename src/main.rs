mod daemon;

use clap::{clap_app, crate_version};

fn main() {
    async_std::task::block_on(run());
}

async fn run() {
    let clap = clap_app!(livebuds => (version:crate_version!())
                         (author:"Jojii S")
                         (about:"Control your Galaxy Buds live from cli")
                         (@arg daemon: -d --daemon "Starts the daemon"))
    .get_matches();

    if clap.is_present("daemon") {
        daemon::run_daemon().await;
        return;
    }

    println!("other commands")
}
