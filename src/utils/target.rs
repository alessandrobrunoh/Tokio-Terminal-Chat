use crate::shared_state::ClientMap;
use crate::utils::error::BoxError;
use tokio::sync::mpsc;

/// Raw target identifier - can be either an ID or a nickname
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) enum Target {
    Id(String),
    Name(String),
    Both(String), // Can be either ID or Name, will be determined during validation
}

/// Target that must be an ID
#[derive(Debug, Clone)]
pub(crate) struct TargetId(pub String);

/// Target that must be a nickname
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct TargetName(pub String);

/// Validated target user with full information
pub(crate) struct ValidatedTarget {
    id: u32,
    nickname: String,
}

impl Target {
    /// Create a Target from command arguments (can be ID or Name)
    pub(crate) fn from_args(args: &str) -> Option<Self> {
        let trimmed = args.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(Target::Both(trimmed.to_string()))
        }
    }

    /// Get the inner string value regardless of variant
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Target::Id(s) | Target::Name(s) | Target::Both(s) => s,
        }
    }
}

impl TargetId {
    /// Create a TargetId from command arguments
    pub(crate) fn from_args(args: &str) -> Option<Self> {
        let trimmed = args.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(TargetId(trimmed.to_string()))
        }
    }
}

impl TargetName {
    /// Create a TargetName from command arguments
    #[allow(dead_code)]
    pub(crate) fn from_args(args: &str) -> Option<Self> {
        let trimmed = args.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(TargetName(trimmed.to_string()))
        }
    }
}

impl ValidatedTarget {
    /// Validate a Target (can be either ID or Name)
    pub(crate) async fn from_target(
        target: &Target,
        tx: &mpsc::Sender<String>,
        clients: &ClientMap,
    ) -> Result<Self, BoxError> {
        match target {
            Target::Id(id_str) => Self::from_id(id_str, tx, clients).await,
            Target::Name(name) => Self::from_name(name, tx, clients).await,
            Target::Both(input) => {
                // Try as ID first, then as name
                if let Ok(target) = Self::from_id(input, tx, clients).await {
                    Ok(target)
                } else {
                    Self::from_name(input, tx, clients).await
                }
            }
        }
    }

    /// Validate a TargetId (must be a numeric ID)
    pub(crate) async fn from_target_id(
        target_id: &TargetId,
        tx: &mpsc::Sender<String>,
        clients: &ClientMap,
    ) -> Result<Self, BoxError> {
        Self::from_id(&target_id.0, tx, clients).await
    }

    /// Validate a TargetName (must be a nickname)
    #[allow(dead_code)]
    pub(crate) async fn from_target_name(
        target_name: &TargetName,
        tx: &mpsc::Sender<String>,
        clients: &ClientMap,
    ) -> Result<Self, BoxError> {
        Self::from_name(&target_name.0, tx, clients).await
    }

    /// Internal: Validate by ID
    async fn from_id(
        id_str: &str,
        tx: &mpsc::Sender<String>,
        clients: &ClientMap,
    ) -> Result<Self, BoxError> {
        // Parse as u32
        let user_id: u32 = match id_str.parse() {
            Ok(id) => id,
            Err(_) => {
                tx.send(format!("Error: Invalid user ID: {}\n", id_str))
                    .await?;
                return Err("Invalid user ID".into());
            }
        };

        // Lookup in clients
        let clients_lock = clients.lock().await;
        let nickname = if let Some(client_state) = clients_lock.get(&user_id) {
            client_state.nickname.clone()
        } else {
            drop(clients_lock);
            tx.send(format!("Error: User with ID {} not found\n", user_id))
                .await?;
            return Err("User not found".into());
        };

        Ok(ValidatedTarget {
            id: user_id,
            nickname,
        })
    }

    /// Internal: Validate by nickname
    async fn from_name(
        name: &str,
        tx: &mpsc::Sender<String>,
        clients: &ClientMap,
    ) -> Result<Self, BoxError> {
        let clients_lock = clients.lock().await;

        // Search for user by nickname
        for (user_id, client_state) in clients_lock.iter() {
            if client_state.nickname.eq_ignore_ascii_case(name) {
                return Ok(ValidatedTarget {
                    id: *user_id,
                    nickname: client_state.nickname.clone(),
                });
            }
        }

        drop(clients_lock);
        tx.send(format!("Error: User '{}' not found\n", name))
            .await?;
        Err("User not found".into())
    }

    /// Get the target's user ID
    pub(crate) fn id(&self) -> u32 {
        self.id
    }

    /// Get the target's nickname
    pub(crate) fn nickname(&self) -> &str {
        &self.nickname
    }

    /// Send a direct message to this target user
    pub(crate) async fn send_message(
        &self,
        clients: &ClientMap,
        message: &str,
    ) -> Result<(), BoxError> {
        let clients_lock = clients.lock().await;
        if let Some(client_state) = clients_lock.get(&self.id) {
            client_state.tx.send(message.to_string()).await?;
        }
        Ok(())
    }

    /// Broadcast a message to all clients
    pub(crate) async fn broadcast_to_all(
        clients: &ClientMap,
        message: &str,
    ) -> Result<(), BoxError> {
        let clients_lock = clients.lock().await;
        for (_, client_state) in clients_lock.iter() {
            let _ = client_state.tx.send(message.to_string()).await;
        }
        Ok(())
    }
}
