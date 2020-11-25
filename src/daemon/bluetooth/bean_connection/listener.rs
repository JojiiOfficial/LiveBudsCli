use super::super::super::{buds_config::Config, buds_info::BudsInfo};
use super::super::{bt_connection_listener::BudsConnection, rfcomm_connector::ConnHandler};
use super::{extended_status_update, status_update, touchpad};

use async_std::io::prelude::*;
use async_std::sync::Mutex;
use galaxy_buds_live_rs::message::{ids, Message};

use std::{process::exit, sync::Arc};

const BUFF_SIZE: usize = 2048;

/// Read buds data
pub async fn start_listen(
    connection: BudsConnection,
    config: Arc<Mutex<Config>>,
    ch: Arc<Mutex<ConnHandler>>,
) {
    let mut stream = connection.socket.get_stream();
    let mut buffer: Vec<u8> = Vec::with_capacity(BUFF_SIZE);

    // Check config
    {
        let mut cfg = config.lock().await;
        if let Err(err) = cfg.load().await {
            eprintln!("{}", err);
            exit(1);
        }
    }

    loop {
        for _ in 0..BUFF_SIZE {
            buffer.push(0);
        }

        let bytes_read = match stream.read(&mut buffer).await {
            Ok(v) => v,
            Err(_) => {
                let mut c = ch.lock().await;
                c.remove_device(connection.addr.as_str()).await;
                return;
            }
        };

        // The received message from the buds
        let message = Message::new(&buffer[0..bytes_read]);

        let connection_handler = ch.lock().await;
        let mut lock = connection_handler.connection_data.lock().await;

        let info = lock
            .data
            .entry(connection.addr.clone())
            .or_insert_with(|| BudsInfo::new(stream.clone(), &connection.addr));

        match message.get_id() {
            ids::TOUCHPAD_ACTION => {
                touchpad::handle_tap(message.into(), info, &config, &connection).await
            }

            ids::STATUS_UPDATED => {
                status_update::handle(message.into(), info, &config, &connection).await
            }

            ids::EXTENDED_STATUS_UPDATED => {
                extended_status_update::handle(message.into(), info);
            }

            _ => continue,
        };
    }
}
