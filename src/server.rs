use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::client::Client;
use crate::shared_state::ClientMap;

pub(crate) struct Server {
    listener: TcpListener,
    clients: ClientMap,
    client_id_counter: Arc<AtomicU32>,
}

impl Server {
    pub(crate) async fn new(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        println!("Server listening on {}", addr);
        Ok(Server {
            listener,
            clients: Arc::new(Mutex::new(HashMap::new())),
            client_id_counter: Arc::new(AtomicU32::new(1)),
        })
    }

    pub(crate) async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let (socket, addr) = self.listener.accept().await?;
            let client_id = self.client_id_counter.fetch_add(1, Ordering::SeqCst);

            println!("New connection from client {}: {}", client_id, addr);

            let client = Client::new(client_id, socket, Arc::clone(&self.clients));

            tokio::spawn(async move {
                client.handle().await;
            });
        }
    }
}
