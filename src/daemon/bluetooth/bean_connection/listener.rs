use super::super::{bt_connection_listener::BudsConnection, rfcomm_connector::ConnHandler};
use super::{
    super::super::{buds_config::Config, buds_info::BudsInfo},
    ambient_mode,
};
use super::{anc, extended_status_update, get_all_data, status_update, touchpad};

use async_std::io::prelude::*;
use async_std::sync::Mutex;
use galaxy_buds_rs::message::debug::GetAllData;
use galaxy_buds_rs::{
    message::{self, ids, Message, Payload},
    model::Model,
};

use std::{process::exit, sync::Arc};

const BUFF_SIZE: usize = 2048;

/// Read buds data
pub async fn start_listen(
    connection: BudsConnection,
    config: Arc<Mutex<Config>>,
    ch: Arc<Mutex<ConnHandler>>,
    model: Model,
) {
    let mut stream = connection.socket.get_stream();
    let mut buffer: Vec<u8> = vec![0u8; BUFF_SIZE];

    // Check config errors
    {
        let mut cfg = config.lock().await;
        if let Err(err) = cfg.load().await {
            eprintln!("{}", err);
            exit(1);
        }
    }

    let mut requested_debug = false;
    let mut first_msg = true;

    loop {
        let bytes_read = match stream.read(&mut buffer).await {
            Ok(v) => v,
            Err(_) => {
                let mut c = ch.lock().await;
                c.remove_device(connection.addr.as_str()).await;
                return;
            }
        };

        // The received message from the buds
        let message = Message::new(&buffer[0..bytes_read], model);

        if !message.is_message() {
            first_msg = true;
            continue;
        }

        // validate crc checksum
        if !message.check_crc() {
            // First received message always throws an CRC error. Since its nothing important we
            // can igonre it. However we don't want and need it to print an error.
            if !first_msg {
                println!("WARNING: CRC failed. Skipping message");
            }
            first_msg = false;
            continue;
        }

        // Use a variable to store whether the connection should be closed at the end of the
        // following scope. This is necessary because the 'lock' can't be borrowed twice at the
        // same time. Yes, I do hate me for this.
        let mut disconnect_afterwards = false;

        {
            let connection_handler = ch.lock().await;
            let mut lock = connection_handler.connection_data.lock().await;

            let info = lock
                .data
                .entry(connection.addr.clone())
                .or_insert_with(|| BudsInfo::new(stream.clone(), &connection.addr, model));

            match message.get_id() {
                ids::TOUCHPAD_ACTION => {
                    if touchpad::handle(message.into(), info, &config, &connection).await {
                        disconnect_afterwards = true;
                    }
                }

                ids::STATUS_UPDATED => {
                    status_update::handle(message.into(), info, &config, &connection).await
                }

                ids::EXTENDED_STATUS_UPDATED => {
                    extended_status_update::handle(message.into(), info);

                    // Respond with set manager
                    stream
                        .write(&message::manager::new(true, 24).get_data())
                        .await
                        .unwrap();
                }

                ids::DEBUG_GET_ALL_DATA => {
                    let dbg_data: Option<GetAllData> = message.into();
                    if let Some(data) = dbg_data {
                        get_all_data::handle(data, info);
                    }
                }

                ids::AMBIENT_MODE_UPDATED => {
                    ambient_mode::handle(message.into(), info);
                }

                ids::NOISE_REDUCTION_MODE_UPDATE => {
                    anc::handle(message.into(), info);
                }

                _ => (),
            };

            // Send debug request at an appropriate interval
            if !requested_debug || info.last_debug.elapsed().unwrap_or_default().as_secs() >= 8 {
                if let Err(err) = info.request_debug_data().await {
                    println!("Error sending debug request {:?}", err);
                }
            }

            if !requested_debug {
                requested_debug = true;
            }
        }

        if first_msg {
            first_msg = false;
        }

        // Disconnect from device
        if disconnect_afterwards {
            println!("Disconnecting from device {}", connection.addr);
            ch.lock().await.remove_device(&connection.addr).await;
            return;
        }
    }
}
