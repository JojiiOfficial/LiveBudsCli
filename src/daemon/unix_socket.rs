use super::bud_connection::BudsInfo;
use super::connection_handler::ConnectionData;

use async_std::{
    io::{prelude::*, BufReader, BufWriter},
    os::unix::net::{UnixListener, UnixStream},
    prelude::*,
    sync::Mutex,
};
use galaxy_buds_live_rs::message::{
    bud_property::{BudProperty, EqualizerType},
    set_noise_reduction,
    simple::new_equalizer,
};
use serde_derive::{Deserialize, Serialize};

use std::{path::Path, sync::Arc};

/// Runs the unix socket which provides the user API
pub async fn run<P: AsRef<Path>>(p: P, cd: Arc<Mutex<ConnectionData>>) {
    let p = p.as_ref();
    let listener = UnixListener::bind(p).await.unwrap();
    let mut incoming = listener.incoming();

    loop {
        for stream in incoming.next().await {
            if let Err(err) = stream {
                // This is fatal so we can exit the program here.
                panic!("Error: {}", err);
            }

            // Start task to handle multiple client socket connections
            async_std::task::spawn(handle_client(stream.unwrap(), Arc::clone(&cd)));
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Request {
    pub cmd: String,
    pub device: Option<String>,
    pub opt_param1: Option<String>,
    pub opt_param2: Option<String>,
    pub opt_param3: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct Response<T>
where
    T: serde::ser::Serialize,
{
    pub status: String,
    pub device: String,
    pub status_message: Option<String>,
    pub payload: Option<T>,
}

impl<T> Response<T>
where
    T: serde::ser::Serialize,
{
    fn new_success<S: AsRef<str>>(device: S, payload: Option<T>) -> Self {
        Self {
            status: "success".to_owned(),
            device: device.as_ref().to_owned(),
            payload,
            status_message: None,
        }
    }

    fn new_error<S: AsRef<str>>(device: String, message: S, payload: Option<T>) -> Self {
        Self {
            status: "error".to_owned(),
            device,
            payload,
            status_message: Some(message.as_ref().to_owned()),
        }
    }
}

/// Handle a unix socket connection
async fn handle_client(stream: UnixStream, cd: Arc<Mutex<ConnectionData>>) {
    let mut read_stream = BufReader::new(&stream);
    let mut write_stream = BufWriter::new(&stream);
    let mut buff = String::new();

    loop {
        // might be still dirty
        buff.clear();

        // Read the request
        if let Err(_) = read_stream.read_line(&mut buff).await {
            return;
        }

        // Parse the request
        let payload = serde_json::from_str::<Request>(buff.as_str());
        let payload = match payload {
            Ok(p) => p,
            Err(_) => return,
        };

        println!("{:?}", payload);

        let connection_data = cd.lock().await;
        let device_addr = &payload.device.clone().unwrap_or_default().clone();

        // Get requested device
        let device_data = match connection_data.get_device(device_addr) {
            Some(v) => v,
            None => {
                // TODO
                // Device not found!
                continue;
            }
        };

        let new_payload;

        match payload.cmd.as_str() {
            "get_status" => {
                new_payload = Response::new_success(device_data.address.clone(), Some(device_data));
            }
            "set_value" => new_payload = set_value(&payload, &device_data).await,
            _ => continue,
        };

        // Respond. Return on error
        if !respond(&new_payload, &mut write_stream).await {
            return;
        }
    }
}

async fn set_value<T>(payload: &Request, device_data: &BudsInfo) -> Response<T>
where
    T: serde::ser::Serialize,
{
    let get_err =
        |msg: &str| -> Response<T> { Response::new_error(device_data.address.clone(), msg, None) };

    // Check required fields set
    if payload.opt_param1.is_none() || payload.opt_param2.is_none() {
        return get_err("Missing parameter");
    }

    let key = payload.opt_param1.clone().unwrap();
    let value = payload.opt_param2.clone().unwrap();

    // Run desired command
    let res = match key.as_str() {
        // Set noise reduction
        "noise_reduction" => {
            let msg = set_noise_reduction::new(str_to_bool(&value));
            device_data.send(msg).await
        }
        // Set EqualizerType command
        "equalizer" => match value.parse::<u8>() {
            Ok(val) => {
                let msg = new_equalizer(EqualizerType::decode(val));
                device_data.send(msg).await
            }
            Err(_) => Err("could not parse value".to_string()),
        },
        _ => return get_err("Invaild key to set to"),
    };

    // Return success or error based on the success of the set command
    if res.is_ok() {
        Response::new_success(device_data.address.clone(), None)
    } else {
        get_err(res.err().unwrap().as_str())
    }
}

fn str_to_bool<S: AsRef<str>>(s: S) -> bool {
    match s.as_ref().to_lowercase().as_str() {
        "1" | "true" | "yes" | "y" => true,
        _ => false,
    }
}

// Respond to client. Return true on success
async fn respond<T>(response: &Response<T>, write_stream: &mut BufWriter<&UnixStream>) -> bool
where
    T: serde::ser::Serialize,
{
    // Write response
    if let Err(err) = write_stream
        .write(serde_json::to_string(response).unwrap().as_bytes())
        .await
    {
        eprintln!("Err: {:?}", err);
        return false;
    }

    // Flush writer
    if let Err(_) = write_stream.flush().await {
        return false;
    }

    true
}
