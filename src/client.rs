use crate::app::GuiApp;
use crate::network::{Request, SystemDataRequest};
use crate::utils::hash::hash;
use std::error::Error;
use std::net::{TcpStream, ToSocketAddrs};

struct ClientID(u64);

impl From<String> for ClientID {
    fn from(value: String) -> Self {
        Self(hash(&value.to_owned()))
    }
}

pub struct Client {
    id: ClientID,
    app: GuiApp,
}

impl Client {
    pub fn new_from_connection(
        name: String,
        server_adress: impl ToSocketAddrs,
    ) -> Result<Self, Box<dyn Error>> {
        let mut stream = TcpStream::connect(server_adress)?;
        let id = name.into();
        let request = SystemDataRequest;
        request.send(&mut stream)?;
        let bodies = request.await_response(&mut stream)?;
        let (app, _) = GuiApp::new_from_bodies(bodies, false)?;
        let client = Client { id, app };
        Ok(client)
    }
}
