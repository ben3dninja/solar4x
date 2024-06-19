use std::error::Error;
use std::io::Result as IoResult;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::task::JoinHandle;

use crate::app::body_data::BodyType;
use crate::app::body_id::BodyID;
use crate::app::{App, TIME_STEP};
use crate::network::{
    NetworkMessage, ServerToClientMessage, SystemDataRequest, SystemDataResponse,
};

pub struct Server {
    app: App,
    listener: TcpListener,
    thread_handles: Vec<JoinHandle<()>>,
    tx: Sender<ServerAction>,
}

pub struct ServerTask {
    stream: TcpStream,
    rx: Receiver<ServerAction>,
    client_address: SocketAddr,
    body_ids: Vec<BodyID>,
}

const CLIENT_NUMBER: usize = 1;
const UPDATE_TICK: Duration = Duration::from_secs(1);

impl Server {
    pub async fn new(address: impl ToSocketAddrs) -> IoResult<Self> {
        let listener = TcpListener::bind(address).await?;
        println!("Server bound to address {}", listener.local_addr()?);
        let app = App::new_from_filter(|data| data.body_type <= BodyType::Moon)?;
        let (tx, _) = broadcast::channel(32);
        Ok(Self {
            app,
            listener,
            thread_handles: vec![],
            tx,
        })
    }

    pub async fn accept_connections(&mut self) -> IoResult<()> {
        let mut counter = 0;
        let body_ids: Vec<BodyID> = self.app.shared_info.bodies.keys().cloned().collect();
        loop {
            let (stream, client_address) = self.listener.accept().await?;
            println!("receiving connection from address {client_address}");
            let rx = self.tx.subscribe();
            let body_ids = body_ids.clone();
            self.thread_handles.push(tokio::spawn(async move {
                ServerTask {
                    stream,
                    rx,
                    client_address,
                    body_ids,
                }
                .run()
                .await
                .expect(&format!(
                    "Server task associated to client {client_address} panicked"
                ));
            }));
            counter += 1;
            if counter == CLIENT_NUMBER {
                break;
            }
        }
        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut previous_time = Instant::now();
        let mut lag = Duration::ZERO;
        let mut tick_counter = Duration::ZERO;
        loop {
            let current_time = Instant::now();
            let elapsed = current_time - previous_time;
            previous_time = current_time;
            tick_counter += elapsed;
            if self.app.time_switch {
                lag += elapsed;
                while lag >= TIME_STEP {
                    self.app.engine.update();
                    self.app.copy_buffer();
                    lag -= TIME_STEP;
                }
            }
            if tick_counter >= UPDATE_TICK {
                self.tx
                    .send(ServerAction::UpdateTime(self.app.engine.time))?;
                tick_counter = Duration::ZERO;
            }
        }
    }
}

impl ServerTask {
    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        SystemDataRequest::read(&mut self.stream).await?;
        SystemDataResponse(self.body_ids.clone())
            .send(&mut self.stream)
            .await?;
        loop {
            if let Ok(action) = self.rx.recv().await {
                match action {
                    ServerAction::UpdateTime(time) => {
                        ServerToClientMessage::UpdateTime {game_time: time}
                            .send(&mut self.stream)
                            .await
                            .unwrap_or_else(|err| {
                                eprintln!(
                                    "Encountered error {err} when sending update time message to client {}", self.client_address
                                );
                            });
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
enum ServerAction {
    UpdateTime(f64),
}
