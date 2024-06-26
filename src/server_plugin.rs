use std::net::IpAddr;

use bevy::prelude::*;
use bevy_quinnet::{
    server::{
        certificate::CertificateRetrievalMode, QuinnetServer, QuinnetServerPlugin,
        ServerEndpointConfiguration,
    },
    shared::ClientId,
};
use bevy_ratatui::error::exit_on_error;

use crate::{
    core_plugin::{start_game, BodiesConfig, CorePlugin},
    engine_plugin::GameTime,
    network::{ServerChannel, ServerMessage},
};

pub struct ServerPlugin {
    pub server_address: (IpAddr, u16),
    pub config: BodiesConfig,
}

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((CorePlugin, QuinnetServerPlugin::default()))
            .add_event::<ClientConnectionEvent>()
            .insert_resource(ServerNetworkInfo {
                server_address: self.server_address,
            })
            .insert_resource(self.config.clone())
            .insert_resource(PeriodicUpdatesTimer(Timer::from_seconds(
                1.,
                TimerMode::Repeating,
            )))
            .insert_resource(Clients::default())
            .add_systems(Startup, start_connection.pipe(exit_on_error))
            .add_systems(
                Update,
                (
                    update_clients,
                    handle_connection_events.pipe(exit_on_error),
                    send_periodic_updates,
                ),
            )
            .add_systems(Startup, start_game);
    }
}

#[derive(Clone, Resource)]
pub struct ServerNetworkInfo {
    pub server_address: (IpAddr, u16),
}

#[derive(Resource, Default)]
struct Clients(Vec<ClientId>);

#[derive(Event)]
enum ClientConnectionEvent {
    Connected(ClientId),
    Disconnected(ClientId),
}

#[derive(Resource)]
struct PeriodicUpdatesTimer(Timer);

fn start_connection(
    mut server: ResMut<QuinnetServer>,
    network_info: Res<ServerNetworkInfo>,
) -> color_eyre::Result<()> {
    let ServerNetworkInfo { server_address } = *network_info;
    server.start_endpoint(
        ServerEndpointConfiguration::from_ip(server_address.0, server_address.1),
        CertificateRetrievalMode::GenerateSelfSigned {
            server_hostname: "rust_space_trading_server".into(),
        },
        ServerChannel::channels_configuration(),
    )?;
    Ok(())
}

fn update_clients(
    mut clients: ResMut<Clients>,
    server: ResMut<QuinnetServer>,
    mut writer: EventWriter<ClientConnectionEvent>,
) {
    let updated_clients = server.endpoint().clients();
    for client in &updated_clients {
        if !clients.0.contains(client) {
            writer.send(ClientConnectionEvent::Connected(*client));
        }
    }
    for client in &clients.0 {
        if !updated_clients.contains(client) {
            writer.send(ClientConnectionEvent::Disconnected(*client));
        }
    }
    clients.0 = updated_clients;
}

fn handle_connection_events(
    mut reader: EventReader<ClientConnectionEvent>,
    mut server: ResMut<QuinnetServer>,
    bodies_config: Res<BodiesConfig>,
) -> color_eyre::Result<()> {
    let endpoint = server.endpoint_mut();
    for event in reader.read() {
        match event {
            ClientConnectionEvent::Connected(id) => {
                println!("Client connected with id {id}");
                endpoint.send_message_on(
                    *id,
                    ServerChannel::Once,
                    ServerMessage::BodiesConfig(bodies_config.clone()),
                )?
            }
            ClientConnectionEvent::Disconnected(id) => {
                println!("Client disconnected with id {id}");
            }
        }
    }
    Ok(())
}

fn send_periodic_updates(
    mut timer: ResMut<PeriodicUpdatesTimer>,
    time: Res<Time>,
    mut server: ResMut<QuinnetServer>,
    game_time: Res<GameTime>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        server.endpoint_mut().try_broadcast_message_on(
            ServerChannel::PeriodicUpdates,
            ServerMessage::UpdateTime {
                game_time: game_time.0,
            },
        );
    }
}
