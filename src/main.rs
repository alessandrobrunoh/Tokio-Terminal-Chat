use crate::server::Server;

mod client;
mod commands;
mod middlewares;
mod server;
mod shared_state;
mod traits;
mod utils;

const PORT: i16 = 8080;
const IP: &str = "127.0.0.1";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = Server::new(&format!("{}:{}", IP, PORT)).await?;
    server.run().await?;
    Ok(())
}
