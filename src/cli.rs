use clap::{Arg, Command, ValueHint};

pub fn build<'a>() -> Command {
    Command::new("earbuds")
        .arg_required_else_help(true)
        .about("ok")
        //.version(crate_version!())
        .author("Jojii S")
        //.help("Control your Galaxy Buds live from cli")
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .global(true)
                .help("Prints informations verbosely"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .global(true)
                .value_parser(["json", "normal"]),
        )
        .arg(
            Arg::new("generator")
                .long("generate")
                .help("Generate completion scripts for a given type of shell")
                .value_parser(["bash", "elvish", "fish", "powershell", "zsh"]),
        )
        .arg(
            Arg::new("daemon")
                .help("Starts the daemon")
                .long("daemon")
                .short('d'),
        )
        .arg(
            Arg::new("no-fork")
                .help("Don't fork the daemon")
                .long("no-fork"),
        )
        .arg(
            Arg::new("kill-daemon")
                .help("Kill the daemon. If used together with -d, the daemon will get restarted")
                .short('k')
                .long("kill-daemon"),
        )
        .arg(
            Arg::new("quiet")
                .help("Don't print extra output")
                .short('q')
                .global(true)
                .long("quiet"),
        )
        .arg(
            Arg::new("device")
                .global(true)
                .help("Specify the device to use")
                .short('s')
                .num_args(1)
                .value_hint(ValueHint::Unknown)
                .long("device"),
        )
        .subcommand(
            Command::new("status")
                .alias("info")
                .about("Display informations for a given device"),
        )
        .subcommand(
            Command::new("set")
                .about("Turn on/off features and control the equalizer setting")
                .arg(Arg::new("key").required(true).num_args(1).value_parser([
                    "equalizer",
                    "anc",
                    "touchpadlock",
                    "touchpad",
                    "ambientsound",
                    "tap-action",
                ]))
                .arg(Arg::new("value").required(true).num_args(1))
                .arg(
                    Arg::new("opt")
                        .help("Provide additional input for some keys")
                        .num_args(1),
                ),
        )
        .subcommand(
            Command::new("enable").about("Turn on a given feature").arg(
                Arg::new("key")
                    .required(true)
                    .num_args(1)
                    .value_parser(["anc", "touchpad"]),
            ),
        )
        .subcommand(
            Command::new("disable")
                .arg_required_else_help(true)
                .about("Turn off a given feature")
                .arg(Arg::new("key").required(true).num_args(1).value_parser([
                    "equalizer",
                    "anc",
                    "touchpad",
                ])),
        )
        .subcommand(
            Command::new("toggle")
                .arg_required_else_help(true)
                .about("Toggle the state of a feature")
                .arg(Arg::new("key").required(true).num_args(1).value_parser([
                    "anc",
                    "touchpadlock",
                    "touchpad",
                ])),
        )
        .subcommand(
            Command::new("config")
                .arg_required_else_help(true)
                .about("Interact with the buds configuration")
                .subcommand(
                    Command::new("set")
                        .arg_required_else_help(true)
                        .about("Set a config value")
                        .arg(Arg::new("key").required(true).num_args(1).value_parser([
                            "auto-pause",
                            "auto-play",
                            "low-battery-notification",
                            "smart-sink",
                        ]))
                        .arg(Arg::new("value").required(true).num_args(1)),
                ),
        )
        // Connect
        .subcommand(Command::new("connect").about("Connect your earbuds"))
        // Disconnect
        .subcommand(Command::new("disconnect").about("Disconnect your earbuds"))
}
