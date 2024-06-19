use crate::app::GuiApp;
use crate::utils::hash::hash;
use std::io::Result;
use std::net::{TcpStream, ToSocketAddrs};

struct ClientID(u64);

impl From<&str> for ClientID {
    fn from(value: &str) -> Self {
        Self(hash(&value.to_owned()))
    }
}

// pub struct Client {
//     id: ClientID,
//     app: GuiApp,
// }

// impl Client {
//     pub fn new_from_connection(name: String, server_adress: impl ToSocketAddrs) -> Result<Self> {
//         let stream = TcpStream::connect(server_adress)?;
//     }
// }
