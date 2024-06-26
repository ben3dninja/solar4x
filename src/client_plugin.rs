use std::net::{IpAddr, Ipv4Addr};

use bevy::prelude::*;
use bevy_quinnet::{
    client::{
        certificate::CertificateVerificationMode, connection::ClientEndpointConfiguration,
        QuinnetClient, QuinnetClientPlugin,
    },
    shared::channels::ChannelsConfiguration,
};
use bevy_ratatui::error::exit_on_error;

use crate::network::{ServerMessage, CHANNEL_TYPES, RELIABLE_CHANNEL, UNRELIABLE_CHANNEL};

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(QuinnetClientPlugin::default())
            .add_systems(Startup, start_connection.pipe(exit_on_error));
    }
}

fn start_connection(mut client: ResMut<QuinnetClient>) -> color_eyre::Result<()> {
    client.open_connection(
        ClientEndpointConfiguration::from_ips(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            6000,
            IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            0,
        ),
        CertificateVerificationMode::SkipVerification,
        ChannelsConfiguration::from_types(CHANNEL_TYPES.into())?,
    )?;
    Ok(())
}

fn handle_server_messages(mut client: ResMut<QuinnetClient>) {
    while let Some((id, message)) = client
        .connection_mut()
        .try_receive_message::<ServerMessage>()
    {
        match message {
            ServerMessage::BodyIDs(bodies) => {}
            ServerMessage::UpdateTime { game_time } => {}
        }
    }
}
