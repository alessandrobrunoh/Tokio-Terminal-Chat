use tokio::sync::mpsc::Sender;

use crate::{shared_state::ClientMap, utils::error::BoxError};

/// Trait that all commands must implement
///
/// Commands are executed in response to client messages and can perform
/// actions such as changing nicknames, sending messages to specific clients,
/// or modifying client state. Each command has access to the client's
/// sender channel, nickname, arguments, shared client map, and client ID.
pub(crate) trait CommandTrait {
    /// Creates a new instance of the command.
    #[allow(unused)]
    fn new() -> Self;
    /// Executes the command.
    async fn execute(
        &self,
        tx: &Sender<String>,
        nickname: &mut String,
        args: &str,
        clients: &ClientMap,
        client_id: u32,
    ) -> Result<(), BoxError>;
}
