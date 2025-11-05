use tokio::sync::mpsc;

use crate::utils::error::BoxError;

use crate::{shared_state::ClientMap, traits::command_trait::CommandTrait};

pub(crate) struct QuitCommand;

impl CommandTrait for QuitCommand {
    /// Create a new instance of the QuitCommand.
    fn new() -> Self {
        QuitCommand
    }
    /// Quit the chat.
    async fn execute(
        &self,
        tx: &mpsc::Sender<String>,
        nickname: &mut String,
        _args: &str,
        _clients: &ClientMap,
        _client_id: u32,
    ) -> Result<(), BoxError> {
        println!("Quitting the chat...");
        tx.send(format!("{} has left the chat.", nickname)).await?;
        *nickname = String::new();
        Ok(())
    }
}
