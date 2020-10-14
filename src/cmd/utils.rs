use clap::ArgMatches;

pub fn get_device_from_app(app: &ArgMatches) -> Option<String> {
    if app.is_present("device") {
        app.value_of("device").map(|s| s.to_owned())
    } else {
        None
    }
}
