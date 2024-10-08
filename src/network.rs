use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use bevy_quinnet::shared::channels::{ChannelId, ChannelType, ChannelsConfiguration};
use serde::{Deserialize, Serialize};

use crate::prelude::BodiesConfig;

pub const SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5000);
pub const CLIENT_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    BodiesConfig(BodiesConfig),
    UpdateTime(u64),
}

#[repr(u8)]
pub enum ServerChannel {
    Once,
    PeriodicUpdates,
}

impl From<ServerChannel> for ChannelId {
    fn from(val: ServerChannel) -> Self {
        val as ChannelId
    }
}
impl ServerChannel {
    pub fn channels_configuration() -> ChannelsConfiguration {
        ChannelsConfiguration::from_types(vec![
            ChannelType::OrderedReliable,
            ChannelType::Unreliable,
        ])
        .unwrap()
    }
}

#[repr(u8)]
pub enum ClientChannel {
    None,
}

impl From<ClientChannel> for ChannelId {
    fn from(val: ClientChannel) -> Self {
        val as ChannelId
    }
}
impl ClientChannel {
    pub fn channels_configuration() -> ChannelsConfiguration {
        ChannelsConfiguration::from_types(vec![
            ChannelType::OrderedReliable,
            // ChannelType::Unreliable,
        ])
        .unwrap()
    }
}
