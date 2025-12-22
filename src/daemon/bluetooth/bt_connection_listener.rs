/*
 * Handles incoming bluetooth connections and
 * forwards connection events to the connector
 */

use bluer::{
    rfcomm::{SocketAddr, Stream},
    AdapterEvent, Address, Device, Uuid,
};

use futures::{stream::FuturesUnordered, StreamExt};
use galaxy_buds_rs::model::Model;
use log::debug;
use tokio::{
    io::AsyncReadExt,
    sync::{Mutex, MutexGuard},
};

use std::sync::mpsc::Sender;
use std::{sync::Arc, time::Duration};

use super::rfcomm_connector::ConnectionEventData;

#[derive(Debug, Clone)]
pub struct SharedStream {
    stream: Arc<Mutex<Stream>>,
}

impl SharedStream {
    pub fn new(stream: Stream) -> Self {
        Self {
            stream: Arc::new(Mutex::new(stream)),
        }
    }

    pub async fn lock_stream(&self) -> MutexGuard<'_, Stream> {
        self.stream.lock().await
    }
}

/// An active connection to a pair of buds
#[derive(Debug)]
pub struct BudsConnection {
    pub addr: Address,
    pub stream: SharedStream,
}

/// Listens for new Bluethooth connections
pub async fn run(sender: Sender<ConnectionEventData>) -> Result<(), String> {
    let session = bluer::Session::new().await.map_err(|i| i.to_string())?;
    let adapter = session.default_adapter().await.map_err(|i| i.to_string())?;
    adapter.set_powered(true).await.map_err(|i| i.to_string())?;

    let mut discovered_stream = adapter
        .discover_devices()
        .await
        .map_err(|i| i.to_string())?;

    while let Some(event) = discovered_stream.next().await {
        match event {
            AdapterEvent::DeviceAdded(address) => {
                let Ok(device) = adapter.device(address.clone()) else {
                    continue;
                };

                let device_id = device
                    .name()
                    .await
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| device.address().to_string());

                if device.is_connected().await != Ok(true) {
                    log::debug!("Found device {device_id} but not connected");
                    continue;
                }

                check_device(&sender, &device).await;
            }
            AdapterEvent::DeviceRemoved(_address) => {
                //
            }
            AdapterEvent::PropertyChanged(_) => {}
        }
    }

    Err("Adapter missing".to_string())
}

// We need this behaivor twice
async fn check_device(sender: &Sender<ConnectionEventData>, device: &Device) {
    let address = device.address();

    let name = device
        .name()
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| address.to_string());

    if !supported_device(&device).await {
        debug!("Not supported: {name}");
        return;
    }

    let model = name_to_model(&name);

    let Some(stream) = discover_factory_service(&device.address()).await else {
        log::info!("Couldn't detect stream for {name:?}");
        return;
    };

    sender
        .send(ConnectionEventData {
            address,
            stream: SharedStream::new(stream),
            model,
        })
        .unwrap();
}

/// Checks whether a device is a pair of buds live
pub async fn supported_device(device: &Device) -> bool {
    const BUDS_UUID: Uuid =
        Uuid::from_bytes([0, 0, 17, 1, 0, 0, 16, 0, 128, 0, 0, 128, 95, 155, 52, 251]);

    device
        .uuids()
        .await
        .ok()
        .flatten()
        .into_iter()
        .flatten()
        .any(|s| s == BUDS_UUID)
}

/// Gives devices model from its name
fn name_to_model(device_name: &str) -> Model {
    let device_name = device_name.trim().to_lowercase();

    if device_name.contains("buds live") {
        Model::BudsLive
    } else if device_name.contains("buds pro") {
        Model::BudsPro
    } else if device_name.contains("buds 2 pro") {
        Model::BudsPro2
    } else if device_name.contains("buds+") {
        Model::BudsPlus
    } else if device_name.contains("buds2") {
        Model::Buds2
    } else if device_name.contains("buds3 pro") {
        Model::Buds3Pro
    } else {
        println!("Couldn't detect model by name ({device_name:?}). Defaulting to \"Buds\"");
        Model::Buds
    }
}

async fn discover_factory_service(addr: &Address) -> Option<Stream> {
    const MAGIC_BYTE_START: u8 = 253;

    let channel_iter = std::iter::once(20).chain(0..20).chain(21..255);

    let mut stream = channel_iter
        .map(|channel| async move {
            let target_sa = SocketAddr::new(addr.clone(), channel);
            let Ok(mut stream) = Stream::connect(target_sa).await else {
                return None;
            };

            let read = tokio::time::timeout(Duration::from_secs(5), stream.read_u8())
                .await
                .ok()
                .transpose()
                .ok()
                .flatten();

            if read == Some(MAGIC_BYTE_START) {
                return Some(stream);
            }

            None
        })
        .collect::<FuturesUnordered<_>>();

    while let Some(next) = stream.next().await {
        if next.is_some() {
            return next;
        }
    }

    None

    // .into_iter()
    // .filter_map(|i| i)
    // .next()
    // .await
}
