use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;

/// Shared state for a connected client
pub(crate) struct SharedClientState {
    pub nickname: String,
    pub tx: mpsc::Sender<String>,
    is_muted: bool,
}

impl SharedClientState {
    /// Create a new client state
    pub fn new(nickname: String, tx: mpsc::Sender<String>) -> Self {
        Self {
            nickname,
            tx,
            is_muted: false,
        }
    }

    /// Check if the client is muted
    pub fn is_muted(&self) -> bool {
        self.is_muted
    }

    /// Mute the client. Returns true if state changed.
    pub fn mute(&mut self) -> bool {
        if !self.is_muted {
            self.is_muted = true;
            true
        } else {
            false
        }
    }

    /// Unmute the client. Returns true if state changed.
    pub fn unmute(&mut self) -> bool {
        if self.is_muted {
            self.is_muted = false;
            true
        } else {
            false
        }
    }

    /// Send a message to this client
    #[allow(dead_code)]
    pub async fn send(
        &self,
        message: impl Into<String>,
    ) -> Result<(), mpsc::error::SendError<String>> {
        self.tx.send(message.into()).await
    }
}

pub(crate) type ClientMap = Arc<tokio::sync::Mutex<HashMap<u32, SharedClientState>>>;
