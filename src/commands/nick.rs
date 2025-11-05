use tokio::sync::mpsc::Sender;

use crate::utils::error::BoxError;

use crate::{shared_state::ClientMap, traits::command_trait::CommandTrait};

pub(crate) struct NicknameCommand;

impl CommandTrait for NicknameCommand {
    /// Create a new instance of the NicknameCommand.
    fn new() -> Self {
        NicknameCommand
    }

    /// Change the user's nickname with full validation.
    async fn execute(
        &self,
        tx: &Sender<String>,
        nickname: &mut String,
        args: &str,
        clients: &ClientMap,
        client_id: u32,
    ) -> Result<(), BoxError> {
        let new_nickname = args.trim();

        // Validation: empty check
        if new_nickname.is_empty() {
            tx.send("Error: Nickname cannot be empty\n".to_string())
                .await?;
            return Ok(());
        }

        // Validation: length check
        if new_nickname.len() > 20 {
            tx.send("Error: Nickname too long (max 20 chars)\n".to_string())
                .await?;
            return Ok(());
        }

        // Validation: character check (alphanumeric + underscore only)
        if !new_nickname
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_')
        {
            tx.send(
                "Error: Nickname can only contain letters, numbers, and underscores\n".to_string(),
            )
            .await?;
            return Ok(());
        }

        // Validation: uniqueness check (case-insensitive)
        let clients_lock = clients.lock().await;
        let is_taken = clients_lock.iter().any(|(id, state)| {
            *id != client_id && state.nickname.eq_ignore_ascii_case(new_nickname)
        });
        drop(clients_lock);

        if is_taken {
            tx.send(format!(
                "Error: Nickname '{}' is already in use\n",
                new_nickname
            ))
            .await?;
            return Ok(());
        }

        // Update nickname
        let old_nickname = std::mem::replace(nickname, new_nickname.to_string());

        // Update in ClientMap
        let mut clients_lock = clients.lock().await;
        if let Some(client_state) = clients_lock.get_mut(&client_id) {
            client_state.nickname = new_nickname.to_string();
        }
        drop(clients_lock);

        // Confirm to user
        tx.send(format!(
            "✅ Nickname changed from '{}' to '{}'\n",
            old_nickname, nickname
        ))
        .await?;

        Ok(())
    }
}
