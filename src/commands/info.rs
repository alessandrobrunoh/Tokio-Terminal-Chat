use tokio::sync::mpsc::Sender;

use crate::{
    shared_state::ClientMap,
    traits::command_trait::CommandTrait,
    utils::error::BoxError,
    utils::target::{Target, ValidatedTarget},
};

pub(crate) struct InfoCommand;

impl CommandTrait for InfoCommand {
    /// Create a new instance of the InfoCommand.
    fn new() -> Self {
        InfoCommand {}
    }

    /// Execute the info command.
    async fn execute(
        &self,
        tx: &Sender<String>,
        _nickname: &mut String,
        args: &str,
        clients: &ClientMap,
        _client_id: u32,
    ) -> Result<(), BoxError> {
        // Parse target - can be either ID or name
        let target_input = Target::from_args(args);

        if target_input.is_none() {
            tx.send(
                "Error: Target cannot be empty\nUsage: /info <user_id or nickname>\n".to_string(),
            )
            .await?;
            return Err("Target cannot be empty".into());
        }

        // Validate target
        let target = ValidatedTarget::from_target(&target_input.unwrap(), tx, clients).await?;

        // Get the client's shared state
        let clients_lock = clients.lock().await;
        let client_state = clients_lock.get(&target.id());

        let message = if let Some(state) = client_state {
            format!(
                "📋 Info for {} (ID: {}):\n  • Nickname: {}\n  • Muted: {}\n",
                target.nickname(),
                target.id(),
                state.nickname,
                if state.is_muted() {
                    "Yes ⚠️"
                } else {
                    "No ✅"
                }
            )
        } else {
            "Error: No information found for user\n".to_string()
        };

        drop(clients_lock);
        tx.send(message).await?;
        Ok(())
    }
}
