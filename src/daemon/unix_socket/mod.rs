pub mod bluetooth_commands;
mod config;
pub mod request_handler;
mod set_value;
pub mod socket;

use serde::{Deserialize, Serialize};

/// Unix connection request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub cmd: String,
    pub device: Option<String>,
    pub opt_param1: Option<String>,
    pub opt_param2: Option<String>,
    pub opt_param3: Option<String>,
}

impl Request {
    pub fn new(cmd: String, device: Option<String>) -> Request {
        Request {
            cmd,
            device,
            opt_param1: None,
            opt_param2: None,
            opt_param3: None,
        }
    }

    /// Get bytes to send for a request
    pub fn sendable(&self) -> serde_json::Result<String> {
        let mut s = serde_json::to_string(self)?;
        s.push('\n');
        Ok(s)
    }
}

/// Unix connection response
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Response<T>
where
    T: serde::ser::Serialize,
{
    pub status: String,
    pub device: String,
    pub status_message: Option<String>,
    pub payload: Option<T>,
}

impl<T> Response<T>
where
    T: serde::ser::Serialize,
{
    /// Create new success response
    fn new_success<S: AsRef<str>>(device_addr: S, payload: Option<T>) -> Self {
        Self {
            status: "success".to_owned(),
            device: device_addr.as_ref().to_owned(),
            payload,
            status_message: None,
        }
    }

    /// Create new Error response
    fn new_error<S: AsRef<str>>(device: String, message: S, payload: Option<T>) -> Self {
        Self {
            status: "error".to_owned(),
            device,
            payload,
            status_message: Some(message.as_ref().to_owned()),
        }
    }

    pub fn from_string<'de>(s: &'de str) -> serde_json::Result<Response<T>>
    where
        T: serde::ser::Serialize + serde::de::Deserialize<'de>,
    {
        serde_json::from_str(s)
    }

    /// return true if response represents a success
    pub fn is_success(&self) -> bool {
        self.status == *"success"
    }
}
