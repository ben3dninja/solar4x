use crate::app::body_id::BodyID;
use crate::app::{AppMessage, GuiApp, TIME_STEP};
use crate::network::{ServerReliableMessage, ServerUnreliableMessage};
use crate::ui::events::UiEvent;
use renet::transport::{ClientAuthentication, NetcodeClientTransport};
// use crate::utils::hash::hash;
use renet::{ConnectionConfig, DefaultChannel, RenetClient};
use std::error::Error;
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

// struct ClientID(u64);

// impl From<String> for ClientID {
//     fn from(value: String) -> Self {
//         Self(hash(&value.to_owned()))
//     }
// }

pub struct Client {
    // id: ClientID,
    app: GuiApp,
    client: RenetClient,
    transport: NetcodeClientTransport,
}

impl Client {
    pub fn new(
        // name: String,
        client_addr: SocketAddr,
        server_addr: SocketAddr,
    ) -> Result<Self, Box<dyn Error>> {
        let mut client = RenetClient::new(ConnectionConfig::default());

        // Setup transport layer
        let socket = UdpSocket::bind(client_addr)?;
        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
        let authentication = ClientAuthentication::Unsecure {
            server_addr,
            client_id: 0,
            user_data: None,
            protocol_id: 0,
        };

        let mut transport = NetcodeClientTransport::new(current_time, authentication, socket)?;

        let tick = Duration::from_millis(5);

        while !client.is_connected() {
            client.update(tick);
            transport.update(tick, &mut client)?;
            thread::sleep(tick);
        }
        let mut i = 0;
        let mut previous = Instant::now();
        let ids = loop {
            if i == 5 {
                break Vec::new();
            }
            let current = Instant::now();
            let elapsed = current - previous;
            previous = current;
            transport.update(elapsed, &mut client)?;
            if let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
                let message = bincode::deserialize::<ServerReliableMessage>(&message)?;
                match message {
                    ServerReliableMessage::BodyIDs(ids) => {
                        break ids.into_iter().map(BodyID::from).collect()
                    }
                }
            }
            i += 1;
            thread::sleep(Duration::from_millis(10));
        };
        let (app, _) = GuiApp::new_from_filter(|data| ids.contains(&data.id), false)?;
        let client = Client {
            app,
            client,
            transport,
        };
        Ok(client)
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut previous_time = Instant::now();
        let mut lag = Duration::ZERO;
        loop {
            let current_time = Instant::now();
            let elapsed = current_time - previous_time;
            previous_time = current_time;
            self.client.update(elapsed);
            self.transport.update(elapsed, &mut self.client)?;
            if self.client.is_connected() {
                while let Some(message) = self.client.receive_message(DefaultChannel::Unreliable) {
                    if let Ok(message) = bincode::deserialize::<ServerUnreliableMessage>(&message) {
                        self.handle_server_message(message);
                    }
                }
            }
            let app = &mut self.app;
            if let Ok(AppMessage::Quit) = app.handle_input() {
                app.ui_event_sender.send(UiEvent::Quit)?;
                if let Some(handle) = app.ui_handle.take() {
                    handle.join().unwrap();
                }
                self.transport.disconnect();
                break;
            }
            if let Ok(err) = app.error_receiver.try_recv() {
                if let Some(handle) = app.ui_handle.take() {
                    handle.join().unwrap();
                }
                return Err(err);
            }

            if app.core.time_switch {
                lag += elapsed;
                while lag >= TIME_STEP {
                    app.core.engine.update();
                    app.core.copy_buffer();
                    lag -= TIME_STEP;
                }
            }
            self.transport.send_packets(&mut self.client)?;
        }
        Ok(())
    }

    pub fn handle_server_message(&mut self, message: ServerUnreliableMessage) {
        match message {
            ServerUnreliableMessage::UpdateTime { game_time } => {
                self.app.core.engine.time = game_time;
            }
        }
    }
}
