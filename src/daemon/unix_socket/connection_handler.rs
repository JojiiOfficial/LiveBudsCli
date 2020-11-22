use super::super::bluetooth::rfcomm_connector::ConnectionData;
use super::super::buds_config::Config;
use super::super::buds_info::BudsInfoInner;
use super::req_executor;
use super::{Request, Response};

use async_std::{
    io::{prelude::*, BufReader, BufWriter},
    os::unix::net::UnixStream,
    sync::{Arc, Mutex},
};

/// Handle a unix socket connection
pub async fn handle_client(
    stream: UnixStream,
    cd: Arc<Mutex<ConnectionData>>,
    config: Arc<Mutex<Config>>,
) {
    let mut read_stream = BufReader::new(&stream);
    let mut write_stream = BufWriter::new(&stream);

    // Read the request
    let mut buff = String::new();
    if read_stream.read_line(&mut buff).await.is_err() {
        return;
    }

    // Parse the request
    let payload = match serde_json::from_str::<Request>(buff.as_str()) {
        Ok(p) => p,
        Err(_) => return,
    };

    let mut connection_data = cd.lock().await;

    // Respond with error if no device is connected and no connect request was made
    if connection_data.get_device_count() == 0 && payload.cmd != "connect" {
        respond(get_err("No connected device found"), &mut write_stream).await;
        return;
    }

    let req_dev_addr = payload.device.clone().unwrap_or_default();
    let device_addr = match connection_data
        .get_device_address(&req_dev_addr, &config)
        .await
    {
        Some(addr) => addr,
        None => {
            respond(get_err("Device not found"), &mut write_stream).await;
            return;
        }
    };

    // Execute the command
    let new_payload = run_payload_cmd(&payload, device_addr, &mut connection_data, config).await;
    if new_payload.is_none() {
        return;
    }

    respond(new_payload.unwrap(), &mut write_stream).await;
}

// Run the requested command
async fn run_payload_cmd(
    payload: &Request,
    device_addr: String,
    connection_data: &mut ConnectionData,
    config: Arc<Mutex<Config>>,
) -> Option<String> {
    Some(match payload.cmd.as_str() {
        "get_status" => {
            let response = Response::new_success(
                &device_addr,
                Some(
                    connection_data
                        .get_device(&device_addr)
                        .unwrap()
                        .inner
                        .clone(),
                ),
            );
            serde_json::to_string(&response).unwrap()
        }
        "set_value" => {
            let mut device = connection_data.get_device_mut(&device_addr).unwrap();
            req_executor::set_buds_value(&payload, &mut device).await
        }
        "toggle_value" => {
            let mut device = connection_data.get_device_mut(&device_addr).unwrap();
            req_executor::toggle_buds_value(&payload, &mut device).await
        }
        "set_config" => req_executor::set_config_value(&payload, device_addr.clone(), config).await,
        "disconnect" | "connect" => {
            req_executor::change_connection_status(device_addr.clone(), payload.cmd == "connect")
                .await
        }

        _ => return None,
    })
}

// Respond to client. Return true on success
async fn respond(response: String, write_stream: &mut BufWriter<&UnixStream>) -> bool {
    // Write response
    if let Err(err) = write_stream.write(response.as_bytes()).await {
        eprintln!("Err: {:?}", err);
        return false;
    }

    // Flush writer
    if write_stream.flush().await.is_err() {
        return false;
    }

    true
}

// Return an serializeable error
pub fn get_err(msg: &str) -> String {
    let err: Response<BudsInfoInner> = Response::new_error("".to_owned(), msg.to_owned(), None);
    serde_json::to_string(&err).unwrap()
}
