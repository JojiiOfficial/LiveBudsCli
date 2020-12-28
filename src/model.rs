use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub enum Model {
    Buds,
    BudsPlus,
    BudsLive,
}

impl From<&str> for Model {
    /// Get the model of Buds by its name
    fn from(device_name: &str) -> Self {
        let device_name = device_name.to_lowercase();

        if device_name.contains("buds live") {
            Model::BudsLive
        } else if device_name.contains("buds+") {
            Model::BudsPlus
        } else {
            Model::Buds
        }
    }
}
