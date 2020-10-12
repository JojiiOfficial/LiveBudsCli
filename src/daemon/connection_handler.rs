use super::bud_connection::BudsConnection;
use std::sync::Arc;

pub struct ConnectionHandler {
    connections: Vec<&'static BudsConnection>,
}

impl ConnectionHandler {
    pub fn new() -> ConnectionHandler {
        ConnectionHandler {
            connections: Vec::new(),
        }
    }

    pub fn add_connection(&mut self, connection: BudsConnection) {
        self.connections.push(&connection);
        async_std::task::spawn(connection.run());

        //self.connections.push(connection);
    }

    pub fn get_conection(&mut self, mac_addr: String) -> Option<&BudsConnection> {
        for i in &self.connections {
            if i.addr == mac_addr {
                return Some(i);
            }
        }

        None
    }
}
