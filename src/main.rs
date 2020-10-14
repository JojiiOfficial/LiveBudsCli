mod daemon;

use clap::{clap_app, crate_version};

use std::env;
use std::path::Path;
use std::process::{exit, Command, Stdio};

const DAEMON_PATH: &str = "/tmp/livebuds.sock";

#[async_std::main]
async fn main() {
    let clap = clap_app!(livebuds => (version:crate_version!())
    (author:"Jojii S")
    (about:"Control your Galaxy Buds live from cli")
    (@arg daemon: -d --daemon "Starts the daemon")
    (@arg quiet: -q --quiet "Don't print extra output")
    )
    .get_matches();

    // run only the daemon if desired
    if clap.is_present("daemon") {
        // Check if a daemon is already running
        if let Err(err) = check_daemon_running(DAEMON_PATH) {
            // Don't print error output if -q is passed
            if !clap.is_present("quiet") {
                eprintln!("{}", err);
            }
            exit(1);
        }

        // run the daemon
        daemon::run_daemon(DAEMON_PATH.to_owned()).await;
        return;
    }

    // We want to start a daemon here if not running
    if check_daemon_running(DAEMON_PATH.to_owned()).is_ok() {
        if !start_background_daemon() {
            exit(1);
        } else if !clap.is_present("quiet") {
            println!("started daemon successfully")
        }
    }
}

/// Start the daemon detached from the current cli
fn start_background_daemon() -> bool {
    let curr_exe = env::current_exe().expect("Couldn't get current executable!");
    let mut cmd = Command::new("nohup");
    let cmd = cmd.arg(curr_exe).arg("-d");
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    let status = cmd.spawn();
    status.is_ok()
}

// Returns an error with a huam friendly message if a daemon is already running
pub fn check_daemon_running<P: AsRef<Path>>(p: P) -> Result<(), String> {
    let p = p.as_ref();

    if !p.exists() {
        return Ok(());
    }

    // Clojure for trying to delete the socket file
    let try_delete = || -> Result<(), String> {
        std::fs::remove_file(p)
            .map_err(|e| format!("Can't delete old socket file {}: {:?}", p.display(), e))?;
        Ok(())
    };

    // Check if the socket file is used by a running program
    if let Ok(files) = ofiles::opath(&p) {
        if files.is_empty() {
            try_delete()?;
        }

        return Err(format!(
            "A daemon is already running: {}",
            files
                .into_iter()
                .map(|i| format!("{:?} ", i))
                .collect::<String>()
        ));
    }

    try_delete()?;
    Ok(())
}
