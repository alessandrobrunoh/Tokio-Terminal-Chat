use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

use crate::commands::Commands;
use crate::middlewares::{MessageContext, MiddlewareChain};
use crate::shared_state::{ClientMap, SharedClientState};

pub(crate) struct Client {
    id: u32,
    nickname: String,
    socket: TcpStream,
    clients: ClientMap,
}

impl Client {
    pub(crate) fn new(id: u32, socket: TcpStream, clients: ClientMap) -> Self {
        let nickname = format!("Client{}", id);
        Client {
            id,
            nickname,
            socket,
            clients,
        }
    }

    pub(crate) async fn handle(self) {
        let Client {
            id,
            mut nickname,
            socket,
            clients,
        } = self;

        let (tx, rx) = mpsc::channel::<String>(10);

        Self::register_client(id, &nickname, &tx, &clients).await;
        let (mut reader, writer) = socket.into_split();
        Self::spawn_writer_task(rx, writer);

        Self::message_loop(id, &mut nickname, &tx, &clients, &mut reader).await;

        Self::disconnect_client(id, &nickname, &clients).await;
    }

    /// Register client in the shared ClientMap
    async fn register_client(
        id: u32,
        nickname: &str,
        tx: &mpsc::Sender<String>,
        clients: &ClientMap,
    ) {
        let client_state = SharedClientState::new(nickname.to_string(), tx.clone());
        clients.lock().await.insert(id, client_state);
    }

    /// Spawn a task to write messages to the client
    fn spawn_writer_task(
        mut rx: mpsc::Receiver<String>,
        mut writer: tokio::net::tcp::OwnedWriteHalf,
    ) {
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if writer.write_all(message.as_bytes()).await.is_err() {
                    break;
                }
            }
        });
    }

    /// Main message reading loop
    async fn message_loop(
        id: u32,
        nickname: &mut String,
        tx: &mpsc::Sender<String>,
        clients: &ClientMap,
        reader: &mut tokio::net::tcp::OwnedReadHalf,
    ) {
        let mut buffer = [0; 1024];
        loop {
            match reader.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => {
                    let message = String::from_utf8_lossy(&buffer[0..n])
                        .trim_end()
                        .to_string();

                    if !Self::handle_message(id, nickname, tx, clients, &message).await {
                        break; // Quit command received
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from client {}: {}", id, e);
                    break;
                }
            }
        }
    }

    /// Handle a single message (command or chat message)
    /// Returns false if client should disconnect
    async fn handle_message(
        id: u32,
        nickname: &mut String,
        tx: &mpsc::Sender<String>,
        clients: &ClientMap,
        message: &str,
    ) -> bool {
        if message.starts_with('/') {
            Self::handle_command(id, nickname, tx, clients, message).await
        } else {
            let mut ctx = MessageContext {
                message: message.to_string(),
                sender_id: id,
                nickname: nickname.clone(),
                clients: clients.clone(),
            };

            // Create and execute middleware chain
            let middleware_chain = MiddlewareChain::default();

            if let Err(e) = middleware_chain.process(&mut ctx).await {
                let _ = tx.send(format!("❌ {}\n", e)).await;
                return true; // Continue but don't send the message
            }

            Self::broadcast_message(id, nickname, clients, &ctx.message).await;
            true
        }
    }

    /// Handle a command. Returns false if client should disconnect
    async fn handle_command(
        id: u32,
        nickname: &mut String,
        tx: &mpsc::Sender<String>,
        clients: &ClientMap,
        command: &str,
    ) -> bool {
        match Commands::handle_command(tx, nickname, command, clients, id).await {
            Ok(should_continue) => should_continue,
            Err(e) => {
                eprintln!("Error handling command for client {}: {}", id, e);
                true // Continue on error
            }
        }
    }

    /// Broadcast a message to all other clients
    async fn broadcast_message(id: u32, nickname: &str, clients: &ClientMap, message: &str) {
        let broadcast_msg = format!("{}: {}\n", nickname, message);
        println!("Broadcasting: {}", broadcast_msg.trim_end());

        let client_txs = {
            let clients_lock = clients.lock().await;
            clients_lock
                .iter()
                .filter(|(client_id, _)| **client_id != id)
                .map(|(_, client_state)| client_state.tx.clone())
                .collect::<Vec<_>>()
        };

        for client_tx in client_txs {
            let _ = client_tx.send(broadcast_msg.clone()).await;
        }
    }

    /// Remove client from the shared ClientMap
    async fn disconnect_client(id: u32, nickname: &str, clients: &ClientMap) {
        println!("Client {} ({}) disconnected.", id, nickname);
        clients.lock().await.remove(&id);
    }
}
