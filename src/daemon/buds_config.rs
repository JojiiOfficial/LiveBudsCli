#![allow(dead_code)]
use serde_derive::{Deserialize, Serialize};

use async_std::fs::{self, File};
use async_std::io::prelude::*;
use async_std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub buds_settings: Vec<BudsConfig>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BudsConfig {
    pub address: String,
    pub low_battery_notification: Option<bool>,
    pub auto_resume_music: Option<bool>,
    pub auto_pause_music: Option<bool>,
    pub auto_sink_change: Option<bool>,
    pub smart_touchpad: Option<bool>,
}

impl Config {
    /// Create a new config object
    pub async fn new() -> Result<Self, String> {
        let config_file = Self::get_config_file().await?;

        let config;

        if !config_file.exists().await {
            config = Self::default();
            config.save().await?;
        } else {
            let conf_data = fs::read_to_string(&config_file)
                .await
                .map_err(|e| e.to_string())?;

            config = toml::from_str(&conf_data).map_err(|e| e.to_string())?;
        }

        Ok(config)
    }

    // Save the config
    pub async fn save(&self) -> Result<(), String> {
        let config_file = Self::get_config_file().await?;

        let s = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        let mut f = File::create(&config_file)
            .await
            .map_err(|e| e.to_string())?;
        f.write_all(&s.as_bytes())
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    // load a config
    pub async fn load(&mut self) -> Result<(), String> {
        let config_file = Self::get_config_file().await?;

        let conf_data = fs::read_to_string(&config_file)
            .await
            .map_err(|e| e.to_string())?;
        *self = toml::from_str(&conf_data).map_err(|e| e.to_string())?;

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

    /// Get configuration for a given device
    pub fn get_device_config_mut(&mut self, address: &str) -> Option<&mut BudsConfig> {
        for elem in &mut self.buds_settings {
            if elem.address.as_str() == address {
                return Some(elem);
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
    pub async fn set_device_config(&mut self, config: BudsConfig) -> Result<(), String> {
        if self.has_device_config(config.address.clone().as_str()) {
            let pos = self.get_device_config_pos(config.address.as_str()).unwrap();
            self.buds_settings[pos] = config;
        } else {
            self.buds_settings.push(config);
        }

        self.save().await
    }

    // Create missing folders and return the config file
    pub async fn get_config_file() -> Result<PathBuf, String> {
        let conf_dir: PathBuf = get_home_dir().unwrap().join(".config").join("livebuds");

        if !conf_dir.exists().await {
            fs::create_dir_all(&conf_dir)
                .await
                .map_err(|e| e.to_string())?;
        }

        Ok(conf_dir.join("config.toml"))
    }
}

pub fn get_home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .and_then(|home| if home.is_empty() { None } else { Some(home) })
        .or_else(|| None)
        .map(PathBuf::from)
}

impl BudsConfig {
    /// Create a new device config
    pub fn new(address: String) -> Self {
        let mut config = Self::default();
        config.address = address;
        config
    }
}

impl BudsConfig {
    pub fn auto_pause(&self) -> bool {
        self.auto_pause_music.unwrap_or(false)
    }

    pub fn auto_play(&self) -> bool {
        self.auto_resume_music.unwrap_or(false)
    }

    pub fn low_battery_notification(&self) -> bool {
        self.low_battery_notification.unwrap_or(false)
    }

    pub fn smart_touchpad(&self) -> bool {
        self.smart_touchpad.unwrap_or(false)
    }

    pub fn auto_sink_change(&self) -> bool {
        self.auto_sink_change.unwrap_or(false)
    }
}
