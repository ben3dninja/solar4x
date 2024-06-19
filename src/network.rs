use std::error::Error;

use bincode::ErrorKind;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::app::body_id::BodyID;

pub trait NetworkMessage: Serialize + DeserializeOwned {
    async fn send(&self, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        Ok(stream.write_all(&bincode::serialize(&self)?).await?)
    }

    async fn read(stream: &mut TcpStream) -> Result<Self, Box<ErrorKind>> {
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await?;
        bincode::deserialize(&buf)
    }
}

#[derive(Serialize, Deserialize)]
pub struct SystemDataRequest;

#[derive(Serialize, Deserialize)]
pub struct SystemDataResponse(pub Vec<BodyID>);

impl NetworkMessage for SystemDataRequest {}
impl NetworkMessage for SystemDataResponse {}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerToClientMessage {
    UpdateTime { game_time: f64 },
}

impl NetworkMessage for ServerToClientMessage {}
