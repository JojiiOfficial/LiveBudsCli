use clap::{crate_version, App, AppSettings, Arg, ValueHint};

pub fn build() -> App<'static> {
    App::new("earbuds")
        .setting(AppSettings::TrailingVarArg)
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(crate_version!())
        .author("Jojii S")
        .about("Control your Galaxy Buds live from cli")
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .global(true)
                .possible_values(&["json", "normal"]),
        )
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
                .global(true)
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
                .about("Display informations for a given device"),
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
                            "equalizer",
                            "anc",
                            "touchpadlock",
                            "touchpad",
                            "tap-action",
                        ]),
                )
                .arg(Arg::new("value").required(true).takes_value(true))
                .arg(
                    Arg::new("opt")
                        .about("Provide additional input for some keys")
                        .takes_value(true),
                ),
        )
        .subcommand(
            App::new("enable")
                .setting(AppSettings::ArgRequiredElseHelp)
                .setting(AppSettings::ColoredHelp)
                .about("Turn off a given features")
                .arg(
                    Arg::new("key")
                        .required(true)
                        .takes_value(true)
                        .possible_values(&["anc", "touchpad"]),
                ),
        )
        .subcommand(
            App::new("disable")
                .setting(AppSettings::ArgRequiredElseHelp)
                .setting(AppSettings::ColoredHelp)
                .about("Turn off a given features")
                .arg(
                    Arg::new("key")
                        .required(true)
                        .takes_value(true)
                        .possible_values(&["equalizer", "anc", "touchpad"]),
                ),
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
                        .possible_values(&["anc", "touchpadlock", "touchpad"]),
                ),
        )
        .subcommand(
            App::new("config")
                .setting(AppSettings::ArgRequiredElseHelp)
                .setting(AppSettings::ColoredHelp)
                .about("Interact with the buds configuration")
                .subcommand(
                    App::new("set")
                        .setting(AppSettings::ArgRequiredElseHelp)
                        .setting(AppSettings::ColoredHelp)
                        .about("Set a config value")
                        .arg(
                            Arg::new("key")
                                .required(true)
                                .takes_value(true)
                                .possible_values(&[
                                    "auto-pause",
                                    "auto-play",
                                    "low-battery-notification",
                                    "smart-sink",
                                ]),
                        )
                        .arg(Arg::new("value").required(true).takes_value(true)),
                ),
        )
        // Connect
        .subcommand(
            App::new("connect")
                .about("Connect your earbuds")
                .setting(AppSettings::ColoredHelp),
        )
        // Disconnect
        .subcommand(
            App::new("disconnect")
                .about("Disconnect your earbuds")
                .setting(AppSettings::ColoredHelp),
        )
}
