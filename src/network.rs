use std::{
    error::Error,
    io::{Read, Write},
};

use bincode::ErrorKind;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::app::body_data::BodyData;

// #[derive(Serialize)]
// pub enum Request {
//     SystemData,
// }
#[derive(Serialize, Deserialize)]
pub struct SystemDataRequest;

pub trait Request: Serialize {
    type Response: DeserializeOwned;
    fn send(&self, stream: &mut impl Write) -> Result<(), Box<dyn Error>> {
        stream.write_all(&bincode::serialize(&self)?)?;
        stream.flush()?;
        Ok(())
    }

    fn await_response(&self, stream: &mut impl Read) -> Result<Self::Response, Box<ErrorKind>> {
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf)?;
        bincode::deserialize(&mut buf)
    }
}

impl Request for SystemDataRequest {
    type Response = Vec<BodyData>;
}
