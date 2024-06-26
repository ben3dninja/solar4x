use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bevy_quinnet::shared::channels::ChannelType;
use serde::{Deserialize, Serialize};

pub const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000);
pub const CLIENT_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);

pub const CHANNEL_TYPES: [ChannelType; 2] = [ChannelType::OrderedReliable, ChannelType::Unreliable];
pub const RELIABLE_CHANNEL: u8 = 0;
pub const UNRELIABLE_CHANNEL: u8 = 1;

#[derive(Serialize, Deserialize)]
pub enum ServerReliableMessage {
    BodyIDs(Vec<u64>),
}

#[derive(Serialize, Deserialize)]
pub enum ServerUnreliableMessage {
    UpdateTime { game_time: f64 },
}

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    BodyIDs(Vec<u64>),
    UpdateTime { game_time: f64 },
}
