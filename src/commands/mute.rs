use tokio::sync::mpsc;

use crate::utils::error::BoxError;

use crate::{
    shared_state::ClientMap,
    traits::command_trait::CommandTrait,
    utils::target::{TargetId, ValidatedTarget},
};

pub(crate) struct MuteCommand;

impl CommandTrait for MuteCommand {
    /// Creates a new instance of the MuteCommand.
    fn new() -> Self {
        MuteCommand
    }

    /// Mute a user
    async fn execute(
        &self,
        tx: &mpsc::Sender<String>,
        _nickname: &mut String,
        args: &str,
        clients: &ClientMap,
        _client_id: u32,
    ) -> Result<(), BoxError> {
        // Parse and validate target
        let target_id = TargetId(args.to_string());
        let target = ValidatedTarget::from_target_id(&target_id, tx, clients).await?;

        // Mute the target user
        let mut clients_lock = clients.lock().await;
        if let Some(client_state) = clients_lock.get_mut(&target.id()) {
            client_state.mute();
        }
        drop(clients_lock);

        // Send notification to the muted user
        let notification = "⚠️  You have been muted by a moderator. You cannot send messages.\n";
        target.send_message(clients, notification).await?;

        // Broadcast to all clients
        let broadcast_msg = format!("🔇 {} has been muted by a moderator.\n", target.nickname());
        println!("Broadcasting: {}", broadcast_msg.trim());
        ValidatedTarget::broadcast_to_all(clients, &broadcast_msg).await?;

        // Confirm to moderator
        let message = format!(
            "✅ Muted user {} (ID: {})\n",
            target.nickname(),
            target.id()
        );
        tx.send(message).await?;

        Ok(())
    }
}

pub(crate) struct UnmuteCommand;

impl CommandTrait for UnmuteCommand {
    /// Creates a new instance of the UnmuteCommand.
    fn new() -> Self {
        UnmuteCommand
    }

    /// Unmute a user
    async fn execute(
        &self,
        tx: &mpsc::Sender<String>,
        _nickname: &mut String,
        args: &str,
        clients: &ClientMap,
        _client_id: u32,
    ) -> Result<(), BoxError> {
        // Parse and validate target
        let target_id = TargetId(args.to_string());
        let target = ValidatedTarget::from_target_id(&target_id, tx, clients).await?;

        // Unmute the target user
        let mut clients_lock = clients.lock().await;
        if let Some(client_state) = clients_lock.get_mut(&target.id()) {
            client_state.unmute();
        }
        drop(clients_lock);

        // Send notification to the unmuted user
        let notification = "✅ You have been unmuted. You can now send messages.\n";
        target.send_message(clients, notification).await?;

        // Broadcast to all clients
        let broadcast_msg = format!("🔊 {} has been unmuted.\n", target.nickname());
        println!("Broadcasting: {}", broadcast_msg.trim());
        ValidatedTarget::broadcast_to_all(clients, &broadcast_msg).await?;

        // Confirm to moderator
        let message = format!(
            "✅ Unmuted user {} (ID: {})\n",
            target.nickname(),
            target.id()
        );
        tx.send(message).await?;

        Ok(())
    }
}
