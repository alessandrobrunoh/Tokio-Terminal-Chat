use tokio::sync::mpsc;

use crate::utils::error::BoxError;

use crate::{shared_state::ClientMap, traits::command_trait::CommandTrait};

pub(crate) struct HelpCommand;

impl CommandTrait for HelpCommand {
    /// Create a new instance of the HelpCommand.
    fn new() -> Self {
        HelpCommand
    }
    /// Display this help message
    async fn execute(
        &self,
        tx: &mpsc::Sender<String>,
        _nickname: &mut String,
        _args: &str,
        _clients: &ClientMap,
        _client_id: u32,
    ) -> Result<(), BoxError> {
        let help_message = "Available commands:\n
            /help - Display this help message\n
            /nickname <new_nickname> - Change your nickname\n
            /quit - Disconnect from the server\n
            /list - List all connected users\n
            /message <user> <message> - Send a private message to a user [NOT_IMPLEMENTED]\n
            /broadcast <message> - Send a message to all connected users [NOT_IMPLEMENTED]\n
            /kick <user> - Kick a user from the server [NOT_IMPLEMENTED]\n
            /ban <user> - Ban a user from the server [NOT_IMPLEMENTED]\n
            /unban <user> - Unban a user from the server [NOT_IMPLEMENTED]\n
            /mute <user> - Mute a user [NOT_IMPLEMENTED]\n
            /unmute <user> - Unmute a user [NOT_IMPLEMENTED]\n
            /history <user> - View a user's chat history [NOT_IMPLEMENTED]\n".to_string();
        tx.send(help_message).await?;
        Ok(())
    }
}
