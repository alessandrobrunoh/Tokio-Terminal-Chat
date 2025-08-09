use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, mpsc};

type ClientMap = Arc<Mutex<HashMap<u32, mpsc::Sender<String>>>>;

struct Server {
    listener: TcpListener,
    clients: ClientMap,
    client_id_counter: Arc<AtomicU32>,
}

impl Server {
    async fn new(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        println!("Il server è in ascolto su {}", addr);
        Ok(Server {
            listener,
            clients: Arc::new(Mutex::new(HashMap::new())),
            client_id_counter: Arc::new(AtomicU32::new(1)),
        })
    }

    async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let (socket, addr) = self.listener.accept().await?;
            let client_id = self.client_id_counter.fetch_add(1, Ordering::SeqCst);

            println!("Nuova connessione dal client {}: {}", client_id, addr);

            let client = Client::new(client_id, socket, Arc::clone(&self.clients));

            tokio::spawn(async move {
                client.handle().await;
            });
        }
    }
}

struct Client {
    id: u32,
    nickname: String,
    socket: TcpStream,
    clients: ClientMap,
}

impl Client {
    fn new(id: u32, socket: TcpStream, clients: ClientMap) -> Self {
        let nickname = format!("Client{}", id);
        Client {
            id,
            nickname,
            socket,
            clients,
        }
    }

    async fn handle_command(tx: &mpsc::Sender<String>, nickname: &mut String, command_line: &str) {
        let parts: Vec<&str> = command_line.trim().splitn(2, ' ').collect();
        let command = parts.get(0).unwrap_or(&"");

        match *command {
            "/nick" => {
                if let Some(new_name) = parts.get(1) {
                    if !new_name.trim().is_empty() {
                        *nickname = new_name.trim().to_string();
                        let response = format!("Nickname cambiato in: {}\n", nickname);
                        let _ = tx.send(response).await;
                    } else {
                        let _ = tx
                            .send("Errore: Il nickname non può essere vuoto.\n".to_string())
                            .await;
                    }
                } else {
                    let _ = tx.send("Uso: /nick <nuovo_nickname>\n".to_string()).await;
                }
            }
            _ => {
                let response = format!("Comando non riconosciuto: {}\n", command);
                let _ = tx.send(response).await;
            }
        }
    }

    async fn handle(self) {
        let Client {
            id,
            mut nickname,
            socket,
            clients,
        } = self;

        let (tx, mut rx) = mpsc::channel::<String>(10);
        clients.lock().await.insert(id, tx.clone());

        let (mut reader, mut writer) = socket.into_split();

        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if writer.write_all(message.as_bytes()).await.is_err() {
                    break;
                }
            }
        });

        let mut buffer = [0; 1024];
        loop {
            match reader.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => {
                    let msg_str = String::from_utf8_lossy(&buffer[0..n]);
                    let msg_trimmed = msg_str.trim_end();

                    if msg_trimmed.starts_with('/') {
                        Client::handle_command(&tx, &mut nickname, msg_trimmed).await;
                    } else {
                        let broadcast_msg = format!("{}: {}\n", nickname, msg_trimmed);
                        println!("Inoltrando: {}", broadcast_msg.trim_end());

                        let clients_lock = clients.lock().await;
                        for (client_id, client_tx) in clients_lock.iter() {
                            if *client_id != id {
                                let _ = client_tx.send(broadcast_msg.clone()).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Errore nella lettura dal client {}: {}", id, e);
                    break;
                }
            }
        }

        println!("Client {} ({}) disconnesso.", id, nickname);
        clients.lock().await.remove(&id);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = Server::new("127.0.0.1:8080").await?;
    server.run().await?;
    Ok(())
}
