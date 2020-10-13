use super::client_handler::ConnectionData;
use async_std::io::{BufReader, BufWriter};
use async_std::os::unix::net::{UnixListener, UnixStream};
use async_std::prelude::*;
use std::sync::{Arc, Mutex};

/// Runs the unix socket which
/// provides the userspace API
pub async fn run(cd: Arc<Mutex<ConnectionData>>) {
    let listener = UnixListener::bind("/tmp/buds-daemon.sock").await.unwrap();
    let mut incoming = listener.incoming();

    loop {
        for stream in incoming.next().await {
            match stream {
                Ok(stream) => {
                    println!("connected");
                    async_std::task::spawn(handle_client(stream, Arc::clone(&cd)));
                }
                Err(err) => {
                    println!("Error: {}", err);
                    break;
                }
            }
        }
    }
}

/// Handle socket connections
async fn handle_client(stream: UnixStream, cd: Arc<Mutex<ConnectionData>>) {
    let mut read_stream = BufReader::new(&stream);
    let mut write_stream = BufWriter::new(&stream);
    let mut buff = String::new();

    loop {
        read_stream.read_line(&mut buff).await.unwrap();

        let v = cd.lock().unwrap().data();

        write_stream
            .write(format!("{:?}", v).as_bytes())
            .await
            .unwrap();

        if let Err(_) = write_stream.flush().await {
            return;
        }

        buff.clear();
    }
}
