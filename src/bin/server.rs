use std::error::Error;

use rust_space_trading::{network::SERVER_ADDR, server::Server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut server = Server::new(SERVER_ADDR)?;
    server.run()
}
