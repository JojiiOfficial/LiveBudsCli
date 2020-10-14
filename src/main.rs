mod cmd;
mod daemon;

use cmd::socket_client::SocketClient;

use clap::{crate_version, App, AppSettings, Arg, ValueHint};
use clap_generate::{
    generate,
    generators::{Bash, Elvish, Fish, PowerShell, Zsh},
    Generator,
};

use std::env;
use std::path::Path;
use std::process::{exit, Command, Stdio};

const DAEMON_PATH: &str = "/tmp/livebuds.sock";

fn build_cli() -> App<'static> {
    App::new("livebuds")
        .setting(AppSettings::TrailingVarArg)
        .version(crate_version!())
        .author("Jojii S")
        .about("Control your Galaxy Buds live from cli")
        .arg(Arg::new("generator").long("generate").possible_values(&[
            "bash",
            "elvish",
            "fish",
            "powershell",
            "zsh",
        ]))
        .arg(
            Arg::new("daemon")
                .about("Starts the daemon")
                .long("daemon")
                .short('d'),
        )
        .arg(
            Arg::new("quiet")
                .about("Don't print extra output")
                .short('q')
                .long("quiet"),
        )
        .arg(
            Arg::new("device")
                .global(true)
                .about("Specify the device to use. Not neccessary if only one device is connected")
                .short('s')
                .takes_value(true)
                .value_hint(ValueHint::Unknown)
                .long("device"),
        )
        .subcommand(
            App::new("status").alias("info").arg(
                Arg::new("output")
                    .short('o')
                    .long("output")
                    .value_hint(ValueHint::Unknown)
                    .possible_values(&["json", "normal"]),
            ),
        )
}

#[async_std::main]
async fn main() {
    let clap = build_cli().get_matches();

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

    if let Some(generator) = clap.value_of("generator") {
        let mut app = build_cli();
        match generator {
            "bash" => print_completions::<Bash>(&mut app),
            "elvish" => print_completions::<Elvish>(&mut app),
            "fish" => print_completions::<Fish>(&mut app),
            "powershell" => print_completions::<PowerShell>(&mut app),
            "zsh" => print_completions::<Zsh>(&mut app),
            _ => println!("Unknown generator"),
        }
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

    let mut socket_client = match SocketClient::new(&DAEMON_PATH) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("Could not connect to daemon: {:?}", err);
            exit(1);
        }
    };

    match clap.subcommand_name() {
        Some("status") => cmd::info::show(&mut socket_client, clap),
        _ => return,
    };
}

fn print_completions<G: Generator>(app: &mut App) {
    generate::<G, _>(app, app.get_name().to_string(), &mut std::io::stdout());
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
