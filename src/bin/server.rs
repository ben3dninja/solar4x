use std::error::Error;

use rust_space_trading::server::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut server = Server::new("127.0.0.1:0").await?;
    server.accept_connections().await?;
    server.run().await
}
