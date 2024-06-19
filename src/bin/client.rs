use std::error::Error;

use rust_space_trading::client::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let port = std::env::args().nth(1).unwrap();
    println!("Received port {port}");
    let mut client = Client::new_from_connection(format!("127.0.0.1:{port}")).await?;
    client.run().await
}
