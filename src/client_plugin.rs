use std::net::{IpAddr, Ipv4Addr};

use bevy::prelude::*;
use bevy_quinnet::client::{
    certificate::CertificateVerificationMode, connection::ClientEndpointConfiguration,
    QuinnetClient, QuinnetClientPlugin,
};
use bevy_ratatui::error::exit_on_error;

use crate::{
    core_plugin::{AppState, BodiesConfig, CorePlugin},
    engine_plugin::GameTime,
    network::{ClientChannel, ServerMessage},
};

use self::explorer_mode::ExplorerPlugin;

pub mod explorer_mode;

#[derive(Default)]
pub struct ClientPlugin {
    pub network_info: ClientNetworkInfo,
    pub explorer_bodies_config: BodiesConfig,
}

impl ClientPlugin {
    pub fn testing(explorer_bodies_config: BodiesConfig) -> Self {
        Self {
            explorer_bodies_config,
            ..Default::default()
        }
    }
}

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((CorePlugin, QuinnetClientPlugin::default()))
            .add_plugins(ExplorerPlugin {
                config: self.explorer_bodies_config.clone(),
            })
            .insert_resource(self.network_info.clone())
            .insert_state(ClientMode::default())
            .add_systems(OnEnter(ClientMode::None), unload)
            .add_systems(
                OnEnter(ClientMode::Multiplayer),
                start_connection.pipe(exit_on_error),
            )
            .add_systems(
                Update,
                handle_server_messages.run_if(in_state(ClientMode::Multiplayer)),
            );
    }
}

#[derive(Default, States, Debug, PartialEq, Eq, Clone, Hash)]
pub enum ClientMode {
    #[default]
    None,
    Singleplayer,
    Multiplayer,
    Explorer,
}

fn unload(mut app_state: ResMut<NextState<AppState>>) {
    app_state.set(AppState::Setup);
}

#[derive(Clone, Resource)]
pub struct ClientNetworkInfo(pub IpAddr, pub u16);
impl Default for ClientNetworkInfo {
    fn default() -> Self {
        Self(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0)
    }
}

#[derive(Clone, Resource)]
pub struct ServerNetworkInfo(pub IpAddr, pub u16);

fn start_connection(
    mut client: ResMut<QuinnetClient>,
    client_info: Res<ClientNetworkInfo>,
    server_info: Res<ServerNetworkInfo>,
) -> color_eyre::Result<()> {
    let ClientNetworkInfo(ca, cp) = *client_info;
    let ServerNetworkInfo(sa, sp) = *server_info;
    client.open_connection(
        ClientEndpointConfiguration::from_ips(sa, sp, ca, cp),
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
                next_state.set(AppState::Loaded);
            }
            ServerMessage::UpdateTime { game_time } => time.0 = game_time,
        }
    }
}
