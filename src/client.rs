use crate::app::body_id::BodyID;
use crate::app::{AppMessage, GuiApp, TIME_STEP};
use crate::network::{
    NetworkMessage, ServerToClientMessage, SystemDataRequest, SystemDataResponse,
};
use crate::ui::events::UiEvent;
// use crate::utils::hash::hash;
use std::error::Error;
use std::time::{Duration, Instant};
use tokio::net::{TcpStream, ToSocketAddrs};

// struct ClientID(u64);

// impl From<String> for ClientID {
//     fn from(value: String) -> Self {
//         Self(hash(&value.to_owned()))
//     }
// }

pub struct Client {
    // id: ClientID,
    app: GuiApp,
    stream: TcpStream,
}

impl Client {
    pub async fn new_from_connection(
        // name: String,
        server_adress: impl ToSocketAddrs,
    ) -> Result<Self, Box<dyn Error>> {
        let mut stream = TcpStream::connect(server_adress).await?;
        // let id = name.into();
        SystemDataRequest.send(&mut stream).await?;
        let ids = SystemDataResponse::read(&mut stream).await?.0;
        println!("Received ids, building app");
        let (app, _) = GuiApp::new_from_filter(|data| ids.contains(&data.id), false)?;
        let client = Client { app, stream };
        Ok(client)
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut previous_time = Instant::now();
        let mut lag = Duration::ZERO;
        loop {
            if let Ok(message) = ServerToClientMessage::read(&mut self.stream).await {
                self.handle_server_message(message).await;
            }
            let app = &mut self.app;
            if let Ok(AppMessage::Quit) = app.handle_input() {
                app.ui_event_sender.send(UiEvent::Quit)?;
                if let Some(handle) = app.ui_handle.take() {
                    handle.join().unwrap();
                }
                break;
            }
            if let Ok(err) = app.error_receiver.try_recv() {
                if let Some(handle) = app.ui_handle.take() {
                    handle.join().unwrap();
                }
                return Err(err);
            }
            let current_time = Instant::now();
            let elapsed = current_time - previous_time;
            previous_time = current_time;
            if app.core.time_switch {
                lag += elapsed;
                while lag >= TIME_STEP {
                    app.core.engine.update();
                    app.core.copy_buffer();
                    lag -= TIME_STEP;
                }
            }
        }
        Ok(())
    }

    pub async fn handle_server_message(&mut self, message: ServerToClientMessage) {
        match message {
            ServerToClientMessage::UpdateTime { game_time } => {
                self.app.core.engine.time = game_time;
            }
        }
    }
}
