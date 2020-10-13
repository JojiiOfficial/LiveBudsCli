use dirs::home_dir;
use serde_derive::{Deserialize, Serialize};

use async_std::fs::{self, File};
use async_std::io::prelude::*;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub buds_settings: Vec<BudsConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BudsConfig {
    pub address: String,
    pub low_battery_notification: bool,
    pub auto_resume_music: bool,
}

impl Config {
    /// Create a new config object
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_file = Self::get_config_file().await?;

        println!("{:#?}", config_file);

        let config;

        if !config_file.exists() {
            config = Self::default();
            config.save().await?;
        } else {
            let conf_data = fs::read_to_string(&config_file).await?;
            config = toml::from_str(&conf_data)?;
        }

        Ok(config)
    }

    // Save the config
    pub async fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_file = Self::get_config_file().await?;

        let s = toml::to_string_pretty(self)?;
        let mut f = File::create(&config_file).await?;
        f.write_all(&s.as_bytes()).await?;

        Ok(())
    }

    /// Get configuration for a given device
    pub fn get_device_config(&self, address: &str) -> Option<&BudsConfig> {
        for i in &self.buds_settings {
            if i.address == *address {
                return Some(i);
            }
        }
        None
    }

    /// Check whether the config has a given device config
    pub fn has_device_config(&self, address: &str) -> bool {
        self.buds_settings.iter().any(|i| i.address == address)
    }

    /// Get the position of a device_config item in the list
    fn get_device_config_pos(&self, address: &str) -> Option<usize> {
        for (i, v) in self.buds_settings.iter().enumerate() {
            if v.address.as_str() == address {
                return Some(i);
            }
        }
        None
    }

    /// Set the config of a specific device. If the config
    /// entry does not exist yet, it will be added
    pub async fn set_device_config(
        &mut self,
        config: BudsConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.has_device_config(config.address.clone().as_str()) {
            let pos = self.get_device_config_pos(config.address.as_str()).unwrap();
            self.buds_settings[pos] = config;
        } else {
            self.buds_settings.push(config);
        }

        self.save().await
    }

    // Create missing folders and return the config file
    pub async fn get_config_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let conf_dir = home_dir().unwrap().join(".config").join("livebuds");
        if !conf_dir.exists() {
            fs::create_dir_all(&conf_dir).await?;
        }
        Ok(conf_dir.join("config.toml"))
    }
}
