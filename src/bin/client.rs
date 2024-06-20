use std::error::Error;

use rust_space_trading::{
    client::Client,
    network::{CLIENT_ADDR, SERVER_ADDR},
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new(CLIENT_ADDR, SERVER_ADDR)?;
    client.run()
}
