mod utils;

use async_std::io::{BufReader, BufWriter};
use async_std::os::unix::net::{UnixListener, UnixStream};
use async_std::prelude::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Request {}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Response {}

pub async fn run_daemon() {
    async_std::task::spawn(start_socket());
    start_bluetooth_listener().await;
}

use blurz::{
    BluetoothDevice,
    BluetoothEvent::{self, Connected},
    BluetoothSession,
};

async fn start_bluetooth_listener() {
    let session = &BluetoothSession::create_session(None).unwrap();

    loop {
        for event in session.incoming(10000).map(BluetoothEvent::from) {
            if event.is_none() {
                continue;
            }
            let event = event.unwrap();

            if let Connected {
                object_path,
                connected,
            } = event
            {
                let device = BluetoothDevice::new(&session, object_path.clone());

                if !utils::is_bt_device_buds_live(&device) {
                    println!("Non buds connected!");
                    continue;
                }

                println!("Buds connected!!!");
            }
        }
    }
}

async fn start_socket() {
    let listener = UnixListener::bind("/tmp/buds-daemon.sock").await.unwrap();
    let mut incoming = listener.incoming();

    loop {
        for stream in incoming.next().await {
            match stream {
                Ok(stream) => {
                    println!("connected");
                    async_std::task::spawn(handle_client(stream));
                }
                Err(err) => {
                    println!("Error: {}", err);
                    break;
                }
            }
        }
    }
}

async fn handle_client(stream: UnixStream) -> u8 {
    let mut read_stream = BufReader::new(&stream);
    let mut write_stream = BufWriter::new(&stream);
    let mut buff = String::new();

    loop {
        read_stream.read_line(&mut buff).await.unwrap();

        write_stream.write(buff.as_bytes()).await.unwrap();
        write_stream.flush().await.unwrap();

        buff.clear();
    }
}
