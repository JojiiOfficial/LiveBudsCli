mod cli;
mod cmd;
mod daemon;
mod daemon_utils;

use clap::{ArgMatches, Command};
use clap_complete::{
    generate,
    shells::{Bash, Elvish, Fish, PowerShell, Zsh},
    Generator,
};
use cmd::socket_client::SocketClient;
use human_panic::setup_panic;

use std::process::exit;

const DAEMON_PATH: &str = "/tmp/earbuds.sock";

#[async_std::main]
async fn main() {
    setup_panic!();

    pretty_env_logger::formatted_builder()
        .filter_module("earbuds", log::LevelFilter::Info)
        .filter_module("galaxy_buds_rs", log::LevelFilter::Info)
        .init();

    let clap = {
        let s = "";
        cli::build(&s).get_matches()
    };

    // Kill daemon if desired and running
    if clap.is_present("kill-daemon")
        && daemon_utils::check_running(DAEMON_PATH.to_owned()).is_err()
    {
        if !daemon_utils::kill(clap.is_present("kill-daemon"), DAEMON_PATH) {
            println!("Couldn't kill daemon");
            return;
        }
    }

    // Run daemon on -k
    if clap.is_present("daemon") {
        // Check if a daemon is already running
        if let Err(err) = daemon_utils::check_running(DAEMON_PATH) {
            // Don't print error output if -q is passed
            if !clap.is_present("quiet") {
                eprintln!("{}", err);
            }
            exit(1);
        }
        // Block if --no-fork is provided
        if clap.is_present("no-fork") {
            daemon::run_daemon(DAEMON_PATH.to_owned()).await;
            return;
        } else
        // Start daemon detached
        if daemon_utils::start() && !clap.is_present("quiet") {
            println!("Daemon started successfully")
        }
        return;
    }
    // Late return to allow a
    // combination of -k and -d
    if clap.is_present("kill-daemon") {
        return;
    }

    // Run generator command if desired
    if let Some(generator) = clap.value_of("generator") {
        generate_completions(generator);
        return;
    }

    // From here we need a running daemon, so ensure one is running
    if daemon_utils::check_running(DAEMON_PATH.to_owned()).is_ok() {
        if !daemon_utils::start() {
            exit(1);
        } else {
            if !clap.is_present("quiet") {
                println!("Daemon started successfully")
            }
            // TODO wait for deamon to be ready
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    }
    run_subcommands(clap);
}

fn run_subcommands(clap: ArgMatches) {
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
        cmd::set_value::set(
            &mut socket_client,
            subcommand,
            false,
            subcommand.value_of("value").unwrap_or_default(),
        );
    }

    // Run disable command
    if let Some(subcommand) = clap.subcommand_matches("disable") {
        cmd::set_value::set(&mut socket_client, subcommand, false, "off");
    }

    // Run enable command
    if let Some(subcommand) = clap.subcommand_matches("enable") {
        cmd::set_value::set(&mut socket_client, subcommand, false, "on");
    }

    // Run toggle command
    if let Some(subcommand) = clap.subcommand_matches("toggle") {
        cmd::set_value::set(
            &mut socket_client,
            subcommand,
            true,
            subcommand.value_of("value").unwrap_or_default(),
        );
    }

    // Run toggle command
    if let Some(config) = clap.subcommand_matches("config") {
        if let Some(set) = config.subcommand_matches("set") {
            cmd::config_set::set(&mut socket_client, set);
        }
    }

    if let Some(subcommand) = clap.subcommand_matches("disconnect") {
        cmd::connection::disconnect(&mut socket_client, subcommand);
    }

    if let Some(subcommand) = clap.subcommand_matches("connect") {
        cmd::connection::connect(&mut socket_client, subcommand);
    }
}

fn generate_completions(generator: &str) {
    let s = "";
    let mut app = cli::build(s);
    match generator {
        "bash" => print_completions(Bash, &mut app),
        "elvish" => print_completions(Elvish, &mut app),
        "fish" => print_completions(Fish, &mut app),
        "powershell" => print_completions(PowerShell, &mut app),
        "zsh" => print_completions(Zsh, &mut app),
        _ => println!("Unknown generator"),
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}
