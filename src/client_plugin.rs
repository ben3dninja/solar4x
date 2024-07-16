use std::net::{IpAddr, Ipv4Addr};

use bevy::prelude::*;
use bevy_quinnet::client::{
    certificate::CertificateVerificationMode, connection::ClientEndpointConfiguration,
    QuinnetClient, QuinnetClientPlugin,
};

use crate::{
    bodies::bodies_config::BodiesConfig,
    core_plugin::{CorePlugin, LoadingState},
    engine_plugin::{EnginePlugin, GameTime},
    main_game::GamePlugin,
    network::{ClientChannel, ServerMessage},
    utils::ecs::exit_on_error_if_app,
};

use self::{explorer_mode::ExplorerPlugin, singleplayer::SingleplayerPlugin};

pub mod explorer_mode;
pub mod singleplayer;

#[derive(Default)]
pub struct ClientPlugin {
    pub network_info: ClientNetworkInfo,
    pub singleplayer_bodies_config: BodiesConfig,
    pub initial_mode: ClientMode,
    pub testing: bool,
}

#[derive(Resource)]
pub struct Testing;

impl ClientPlugin {
    pub fn testing() -> Self {
        Self {
            testing: true,
            ..Default::default()
        }
    }

    pub fn with_bodies(self, singleplayer_bodies_config: BodiesConfig) -> Self {
        Self {
            singleplayer_bodies_config,
            ..self
        }
    }

    pub fn in_mode(self, initial_mode: ClientMode) -> Self {
        Self {
            initial_mode,
            ..self
        }
    }
}

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        if self.testing {
            app.insert_resource(Testing);
        }
        app.add_plugins((CorePlugin, QuinnetClientPlugin::default(), EnginePlugin))
            .add_plugins(ExplorerPlugin(self.singleplayer_bodies_config.clone()))
            .add_plugins(GamePlugin {
                testing: self.testing,
            })
            .add_plugins(SingleplayerPlugin(self.singleplayer_bodies_config.clone()))
            .insert_resource(self.network_info.clone())
            .insert_state(self.initial_mode)
            .add_systems(
                OnEnter(ClientMode::None),
                unload.run_if(in_state(LoadingState::Loaded)),
            )
            .add_systems(
                OnEnter(ClientMode::Multiplayer),
                start_connection.pipe(exit_on_error_if_app),
            )
            .add_systems(
                Update,
                handle_server_messages.run_if(in_state(ClientMode::Multiplayer)),
            );
    }
}

#[derive(Default, States, Debug, PartialEq, Eq, Clone, Hash, Copy)]
pub enum ClientMode {
    #[default]
    None,
    Singleplayer,
    Multiplayer,
    Explorer,
}

fn unload(mut app_state: ResMut<NextState<LoadingState>>) {
    app_state.set(LoadingState::Unloading);
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
    mut next_state: ResMut<NextState<LoadingState>>,
) {
    while let Some((_, message)) = client
        .connection_mut()
        .try_receive_message::<ServerMessage>()
    {
        match message {
            ServerMessage::BodiesConfig(bodies) => {
                commands.insert_resource(bodies);
                next_state.set(LoadingState::Loaded);
            }
            ServerMessage::UpdateTime(simtick) => time.simtick = simtick,
        }
    }
}
