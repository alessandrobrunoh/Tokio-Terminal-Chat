use tokio::sync::mpsc;

use crate::utils::error::BoxError;

use crate::{shared_state::ClientMap, traits::command_trait::CommandTrait};

pub(crate) struct ListCommand;

impl CommandTrait for ListCommand {
    /// Creates a new instance of the ListCommand.
    fn new() -> Self {
        ListCommand
    }

    /// Lists connected users.
    async fn execute(
        &self,
        tx: &mpsc::Sender<String>,
        _nickname: &mut String,
        _args: &str,
        clients: &ClientMap,
        _client_id: u32,
    ) -> Result<(), BoxError> {
        let clients_lock = clients.lock().await;
        let count = clients_lock.len();

        let mut list_message = format!("Connected users ({}):\n", count);

        if count == 0 {
            list_message.push_str("(No users currently connected)\n");
        } else {
            for (id, client_state) in clients_lock.iter() {
                list_message.push_str(&format!("  - {} (ID: {})\n", client_state.nickname, id));
            }
        }

        drop(clients_lock);

        tx.send(list_message).await?;
        Ok(())
    }
}
