use tokio::{net::UnixListener, sync::Mutex};

use super::super::bluetooth::rfcomm_connector::ConnectionData;
use super::super::buds_config::Config;
use super::request_handler;

use std::{path::Path, sync::Arc};

/// Runs the unix socket which provides the user API
pub async fn run<P: AsRef<Path>>(p: P, cd: Arc<Mutex<ConnectionData>>, config: Arc<Mutex<Config>>) {
    let listener = UnixListener::bind(p.as_ref()).unwrap();

    loop {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::task::spawn(request_handler::handle_client(
                stream,
                cd.clone(),
                Arc::clone(&config),
            ));
        }
    }
}
