mod daemon;

use clap::{clap_app, crate_version};

// Run the whole application
async fn run(c: clap::ArgMatches<'static>) {
    // run only the daemon if desired
    if c.is_present("daemon") {
        daemon::run_daemon().await;
        return;
    }

    println!("other commands")
}

fn main() {
    let clap = clap_app!(livebuds => (version:crate_version!())
                         (author:"Jojii S")
                         (about:"Control your Galaxy Buds live from cli")
                         (@arg daemon: -d --daemon "Starts the daemon"))
    .get_matches();

    async_std::task::block_on(run(clap));
}
