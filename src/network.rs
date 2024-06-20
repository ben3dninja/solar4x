use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use serde::{Deserialize, Serialize};

pub const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000);
pub const CLIENT_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);

#[derive(Serialize, Deserialize)]
pub enum ServerReliableMessage {
    BodyIDs(Vec<u64>),
}

#[derive(Serialize, Deserialize)]
pub enum ServerUnreliableMessage {
    UpdateTime { game_time: f64 },
}
