use renet::transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
use renet::{ConnectionConfig, DefaultChannel, RenetServer, ServerEvent};
use std::error::Error;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::time::{Duration, Instant, SystemTime};

use crate::app::body_data::BodyType;
use crate::app::{App, TIME_STEP};
use crate::network::{ServerReliableMessage, ServerUnreliableMessage};

pub struct Server {
    app: App,
    net: (RenetServer, NetcodeServerTransport),
}

const UPDATE_TICK: Duration = Duration::from_secs(1);

impl Server {
    pub fn new(address: SocketAddr) -> Result<Self, Box<dyn Error>> {
        let server = RenetServer::new(ConnectionConfig::default());

        let socket = UdpSocket::bind(address)?;
        let server_config = ServerConfig {
            current_time: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?,
            max_clients: 64,
            protocol_id: 0,
            public_addresses: vec![address],
            authentication: ServerAuthentication::Unsecure,
        };
        let transport = NetcodeServerTransport::new(server_config, socket)?;

        let app = App::new_from_filter(|data| data.body_type <= BodyType::Moon)?;
        let net = (server, transport);
        Ok(Self { app, net })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut previous_time = Instant::now();
        let mut lag = Duration::ZERO;
        let mut tick_counter = Duration::ZERO;
        let body_ids: Vec<u64> = self
            .app
            .shared_info
            .bodies
            .keys()
            .map(|id| id.into())
            .collect();
        let (server, transport) = &mut self.net;
        println!("Running server");
        loop {
            let current_time = Instant::now();
            let elapsed = current_time - previous_time;
            previous_time = current_time;
            tick_counter += elapsed;
            server.update(elapsed);
            transport.update(elapsed, server)?;
            while let Some(event) = server.get_event() {
                match event {
                    ServerEvent::ClientConnected { client_id } => {
                        println!("Client {client_id} connected");
                        server.send_message(
                            client_id,
                            DefaultChannel::ReliableOrdered,
                            bincode::serialize(&ServerReliableMessage::BodyIDs(body_ids.clone()))?,
                        )
                    }
                    ServerEvent::ClientDisconnected { client_id, reason } => {
                        println!("Client {client_id} disconnected: {reason}");
                    }
                }
            }

            // Receive message from channel
            // for client_id in server.clients_id() {
            //     // The enum DefaultChannel describe the channels used by the default configuration
            //     while let Some(message) =
            //         server.receive_message(client_id, DefaultChannel::ReliableOrdered)
            //     {
            //         // Handle received message
            //         if let Ok(message) = bincode::deserialize::<ClientMessage>(&message) {}
            //     }
            // }
            if self.app.time_switch {
                lag += elapsed;
                while lag >= TIME_STEP {
                    self.app.engine.update();
                    self.app.copy_buffer();
                    lag -= TIME_STEP;
                }
            }
            if tick_counter >= UPDATE_TICK {
                server.broadcast_message(
                    DefaultChannel::Unreliable,
                    bincode::serialize(&ServerUnreliableMessage::UpdateTime {
                        game_time: self.app.engine.time,
                    })?,
                );
                tick_counter = Duration::ZERO;
            }
            transport.send_packets(server);
        }
    }
}
