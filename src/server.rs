use std::io::Result;
use std::net::{TcpListener, ToSocketAddrs};
use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;

use crate::app::body_data::BodyType;
use crate::app::{App, AppError};

// pub struct Server {
//     app: App,
//     thread_handles: Vec<JoinHandle<()>>,
//     error_receiver: Receiver<AppError>,
// }

// impl Server {
//     pub fn new(address: impl ToSocketAddrs) -> Result<Self> {
//         let listener = TcpListener::bind(address)?;
//         let app = App::new_from_filter(|data| data.body_type <= BodyType::Moon)?;
//         Ok(Self { app })
//     }
// }
