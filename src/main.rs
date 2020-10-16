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
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(crate_version!())
        .author("Jojii S")
        .about("Control your Galaxy Buds live from cli")
        .arg(
            Arg::new("generator")
                .long("generate")
                .about("Generate completion scripts for a given type of shell")
                .possible_values(&["bash", "elvish", "fish", "powershell", "zsh"]),
        )
        .arg(
            Arg::new("daemon")
                .about("Starts the daemon")
                .long("daemon")
                .short('d'),
        )
        .arg(
            Arg::new("no-fork")
                .about("Don't fork the daemon")
                .long("no-fork"),
        )
        .arg(
            Arg::new("kill-daemon")
                .about("Kill the daemon. If used together with -d, the daemon will get restarted")
                .short('k')
                .long("kill-daemon"),
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
                .about("Specify the device to use")
                .short('s')
                .takes_value(true)
                .value_hint(ValueHint::Unknown)
                .long("device"),
        )
        .subcommand(
            App::new("status")
                .setting(AppSettings::ColoredHelp)
                .alias("info")
                .about("Display informations for a given device")
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_hint(ValueHint::Unknown)
                        .possible_values(&["json", "normal"]),
                ),
        )
        .subcommand(
            App::new("set")
                .setting(AppSettings::ArgRequiredElseHelp)
                .setting(AppSettings::ColoredHelp)
                .about("Turn on/off features and control the equalizer setting")
                .arg(
                    Arg::new("key")
                        .required(true)
                        .takes_value(true)
                        .possible_values(&[
                            "eq",
                            "equalizer",
                            "equalizer-type",
                            "equalizertype",
                            "nc",
                            "anc",
                            "noise-reduction",
                            "noisereduction",
                            "touchpadlock",
                            "tpl",
                            "touchpad",
                        ]),
                )
                .arg(Arg::new("value").required(true).takes_value(true)),
        )
        .subcommand(
            App::new("toggle")
                .setting(AppSettings::ArgRequiredElseHelp)
                .setting(AppSettings::ColoredHelp)
                .about("Toggle the state of a feature")
                .arg(
                    Arg::new("key")
                        .required(true)
                        .takes_value(true)
                        .possible_values(&[
                            "nc",
                            "anc",
                            "noise-reduction",
                            "noisereduction",
                            "touchpadlock",
                            "tpl",
                            "touchpad",
                        ]),
                ),
        )
}

#[async_std::main]
async fn main() {
    let clap = build_cli().get_matches();

    let kill_daemon = clap.is_present("kill-daemon");
    let quiet = clap.is_present("quiet");

    if kill_daemon {
        if check_daemon_running(DAEMON_PATH.to_owned()).is_err() {
            let pids = ofiles::opath(DAEMON_PATH);
            if let Ok(pids) = pids {
                let u: u32 = (*pids.get(0).unwrap()).into();
                if let Err(err) = nix::sys::signal::kill(
                    nix::unistd::Pid::from_raw(u as i32),
                    nix::sys::signal::SIGTERM,
                ) {
                    eprintln!("Error killing process: {:?}", err);
                    exit(1);
                } else if !quiet {
                    println!("Daemon exited!");
                }

                // Hacky way not to display annoying cargo warnings
                try_delete_socket(DAEMON_PATH).unwrap_or_default();
            }
        }
    }

    // run only the daemon if desired
    if clap.is_present("daemon") {
        // Check if a daemon is already running
        if let Err(err) = check_daemon_running(DAEMON_PATH) {
            // Don't print error output if -q is passed
            if !quiet {
                eprintln!("{}", err);
            }
            exit(1);
        }

        // If no-fork is provided, keep daemon blocking
        if clap.is_present("no-fork") {
            daemon::run_daemon(DAEMON_PATH.to_owned()).await;
            return;
        }

        // run the daemon in the background
        if start_background_daemon() && !quiet {
            println!("Daemon started successfully")
        }
        return;
    }

    // Late return to allow the daemon to
    // get restarted as well
    if kill_daemon {
        return;
    }

    // Run generator command if desired
    if let Some(generator) = clap.value_of("generator") {
        generate_completions(generator);
        return;
    }

    // From here we need a running daemon, so ensure one is running
    if check_daemon_running(DAEMON_PATH.to_owned()).is_ok() {
        if !start_background_daemon() {
            exit(1);
        } else {
            if !quiet {
                println!("Daemon started successfully")
            }

            // TODO wait for deamon to be ready
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    }

    // Create a new daemon connection client
    let mut socket_client = match SocketClient::new(&DAEMON_PATH) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("Could not connect to daemon: {:?}", err);
            exit(1);
        }
    };

    // Run status command
    if let Some(subcommand) = clap.subcommand_matches("status") {
        cmd::info::show(&mut socket_client, subcommand);
    }

    // Run set command
    if let Some(subcommand) = clap.subcommand_matches("set") {
        cmd::value::set(&mut socket_client, subcommand, false);
    }

    // Run toggle command
    if let Some(subcommand) = clap.subcommand_matches("toggle") {
        cmd::value::set(&mut socket_client, subcommand, true);
    }
}

fn generate_completions(generator: &str) {
    let mut app = build_cli();
    match generator {
        "bash" => print_completions::<Bash>(&mut app),
        "elvish" => print_completions::<Elvish>(&mut app),
        "fish" => print_completions::<Fish>(&mut app),
        "powershell" => print_completions::<PowerShell>(&mut app),
        "zsh" => print_completions::<Zsh>(&mut app),
        _ => println!("Unknown generator"),
    }
}

fn print_completions<G: Generator>(app: &mut App) {
    generate::<G, _>(app, app.get_name().to_string(), &mut std::io::stdout());
}

/// Start the daemon detached from the current cli
fn start_background_daemon() -> bool {
    let curr_exe = env::current_exe().expect("Couldn't get current executable!");
    let mut cmd = Command::new("nohup");
    let cmd = cmd.arg(curr_exe).arg("-d").arg("--no-fork").arg("-q");
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    let status = cmd.spawn();
    status.is_ok()
}

/// Try to delete the socket file
fn try_delete_socket<P: AsRef<Path>>(p: P) -> Result<(), String> {
    std::fs::remove_file(p.as_ref()).map_err(|e| {
        format!(
            "Can't delete old socket file {}: {:?}",
            p.as_ref().display(),
            e
        )
    })?;
    Ok(())
}

// Returns an error with a huam friendly message if a daemon is already running
pub fn check_daemon_running<P: AsRef<Path>>(p: P) -> Result<(), String> {
    let p = p.as_ref();

    if !p.exists() {
        return Ok(());
    }

    // Check if the socket file is used by a running program
    if let Ok(files) = ofiles::opath(&p) {
        if files.is_empty() {
            try_delete_socket(p)?;
        }

        return Err(format!(
            "A daemon is already running: {}",
            files
                .into_iter()
                .map(|i| format!("{:?} ", i))
                .collect::<String>()
        ));
    }

    try_delete_socket(p)?;
    Ok(())
}
