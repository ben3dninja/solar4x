use std::net::IpAddr;

use bevy::prelude::*;
use bevy_quinnet::client::{
    certificate::CertificateVerificationMode, connection::ClientEndpointConfiguration,
    QuinnetClient, QuinnetClientPlugin,
};
use bevy_ratatui::error::exit_on_error;

use crate::{
    core_plugin::{AppState, BodiesConfig, CorePlugin},
    engine_plugin::GameTime,
    network::{ClientChannel, ServerChannel, ServerMessage},
};

pub struct ClientPlugin(pub ClientNetworkInfo);

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((CorePlugin, QuinnetClientPlugin::default()))
            .insert_resource(self.0.clone())
            .add_systems(Startup, start_connection.pipe(exit_on_error))
            .add_systems(Update, handle_server_messages);
    }
}

#[derive(Clone, Resource)]
pub struct ClientNetworkInfo {
    pub server_address: (IpAddr, u16),
    pub client_address: (IpAddr, u16),
}

fn start_connection(
    mut client: ResMut<QuinnetClient>,
    network_info: Res<ClientNetworkInfo>,
) -> color_eyre::Result<()> {
    let ClientNetworkInfo {
        server_address,
        client_address,
    } = *network_info;
    client.open_connection(
        ClientEndpointConfiguration::from_ips(
            server_address.0,
            server_address.1,
            client_address.0,
            client_address.1,
        ),
        CertificateVerificationMode::SkipVerification,
        ClientChannel::channels_configuration(),
    )?;
    Ok(())
}

fn handle_server_messages(
    mut client: ResMut<QuinnetClient>,
    mut commands: Commands,
    mut time: ResMut<GameTime>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    while let Some((_, message)) = client
        .connection_mut()
        .try_receive_message::<ServerMessage>()
    {
        match message {
            ServerMessage::BodiesConfig(bodies) => {
                commands.insert_resource(bodies);
                next_state.set(AppState::Game);
            }
            ServerMessage::UpdateTime { game_time } => time.0 = game_time,
        }
    }
}
