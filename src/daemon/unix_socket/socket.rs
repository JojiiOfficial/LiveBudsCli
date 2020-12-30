use super::super::bluetooth::rfcomm_connector::ConnectionData;
use super::super::buds_config::Config;
use super::request_handler;

use async_std::{os::unix::net::UnixListener, prelude::*, sync::Mutex};

use std::{path::Path, sync::Arc};

/// Runs the unix socket which provides the user API
pub async fn run<P: AsRef<Path>>(p: P, cd: Arc<Mutex<ConnectionData>>, config: Arc<Mutex<Config>>) {
    let listener = UnixListener::bind(p.as_ref()).await.unwrap();
    let mut incoming = listener.incoming();

    loop {
        while let Some(stream) = incoming.next().await {
            async_std::task::spawn(request_handler::handle_client(
                stream.unwrap(),
                Arc::clone(&cd),
                Arc::clone(&config),
            ));
        }
    }
}
