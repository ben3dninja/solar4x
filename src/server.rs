use std::io::Result;
use std::net::{SocketAddr, TcpListener, ToSocketAddrs};

use crate::app::App;

pub struct Server {
    app: App,
    address: SocketAddr,
    pub engine: Engine,
    pub shared_info: Arc<SystemInfo>,
    pub current_map: Arc<Mutex<GlobalMap>>,
    pub next_map: Arc<Mutex<GlobalMap>>,
    pub time_switch: bool,
    thread_handles: Vec<JoinHandle<()>>,
    error_receiver: Receiver<AppError>,
}

impl Server {
    pub fn new(address: impl ToSocketAddrs) -> Result<Self> {
        let listener = TcpListener::bind(address)?;
        let app = App::new_simple(true, false);
        Ok(Self {
            app:
        })
    }
}
