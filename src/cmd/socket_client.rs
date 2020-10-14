use std::error::Error;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;
use std::path::Path;

use crate::daemon::bud_connection::BudsInfoInner;
use crate::daemon::unix_request_handler::{Request, Response};

pub struct SocketClient {
    #[allow(dead_code)]
    path: String,
    socket: UnixStream,
}

impl SocketClient {
    // Create a new SocketClient
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            path: path.as_ref().to_str().unwrap().to_owned(),
            socket: UnixStream::connect(path)?,
        })
    }

    /// Do a request to the daemon
    pub fn do_request(
        &mut self,
        request: Request,
    ) -> Result<Response<BudsInfoInner>, Box<dyn Error>> {
        let mut stream = &self.socket;

        // send request
        stream.write_all(request.sendable()?.as_bytes())?;
        stream.flush()?;

        // wait for response
        let mut response = String::new();
        stream.read_to_string(&mut response)?;

        Ok(Response::from_string(response.as_str())?)
    }
}

pub fn new_status_request(device: Option<String>) -> Request {
    Request::new("get_status".to_owned(), device)
}
