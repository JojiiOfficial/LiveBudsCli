use super::connection_handler::ConnectionData;

use async_std::{
    io::{prelude::*, BufReader, BufWriter},
    os::unix::net::{UnixListener, UnixStream},
    prelude::*,
    sync::Mutex,
};
use galaxy_buds_live_rs::message::{set_noise_reduction, Payload};
use ofiles;

use std::path::Path;
use std::process::exit;
use std::sync::Arc;

/// Runs the unix socket which
/// provides the userspace API
pub async fn run(cd: Arc<Mutex<ConnectionData>>) {
    let p = Path::new("/tmp/buds-daemon.sock");
    if check_daemon_running(p) {
        exit(1);
    }

    let listener = UnixListener::bind(p).await.unwrap();
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

pub fn check_daemon_running<P: AsRef<Path>>(p: P) -> bool {
    let p = p.as_ref();

    if !p.exists() {
        return false;
    }

    if let Ok(files) = ofiles::opath(&p) {
        if files.len() == 0 {
            std::fs::remove_file(p)
                .expect(format!("Can't delete old socket file: {}", p.display()).as_str());
            return false;
        }

        println!(
            "A daemon is already running: {}",
            files
                .into_iter()
                .map(|i| format!("{:?} ", i))
                .collect::<String>()
        );
    } else {
        // if no proc found, try to delete the socket!
        std::fs::remove_file(p)
            .expect(format!("Can't delete old socket file: {}", p.display()).as_str());
        return false;
    }

    true
}

/// Handle unix socket connections
async fn handle_client(stream: UnixStream, cd: Arc<Mutex<ConnectionData>>) {
    let mut read_stream = BufReader::new(&stream);
    let mut write_stream = BufWriter::new(&stream);
    let mut buff = String::new();

    loop {
        buff.clear();

        read_stream.read_line(&mut buff).await.unwrap();
        let locked = cd.lock().await;
        let info = locked.get_first_device().unwrap();

        if buff == "a\n" {
            let mut v = locked.get_first_stream();
            let send_msg = set_noise_reduction::new(true);
            v.write(&send_msg.to_byte_array()).await.unwrap();
            continue;
        }

        let v = info;
        write_stream
            .write(format!("{:?}", v).as_bytes())
            .await
            .unwrap();

        if let Err(_) = write_stream.flush().await {
            return;
        }
    }
}
