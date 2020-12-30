use crate::daemon::unix_socket::Response;
use clap::ArgMatches;

// return ture if user wants the data in json
pub fn print_as_json(app: &ArgMatches) -> bool {
    app.is_present("output") && app.value_of("output").unwrap() == "json"
}

// Get the device from ArgMatches or none
pub fn get_device_from_app(app: &ArgMatches) -> Option<String> {
    if app.is_present("device") {
        app.value_of("device").map(|s| s.to_owned())
    } else {
        None
    }
}

/// Unwrap a response
pub fn unwrap_response<T>(resp: &Response<T>) -> Option<T>
where
    T: serde::ser::Serialize + Clone,
{
    if !resp.is_success() {
        println!("{}", &resp.status_message.clone().unwrap());
        std::process::exit(1);
    }

    resp.payload.clone()
}

/// Returns true if passed 'input' is parsable to an i32
pub fn is_number<S: AsRef<str>>(input: S) -> bool {
    input.as_ref().parse::<i32>().is_ok()
}
