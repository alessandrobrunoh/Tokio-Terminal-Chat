use crate::{
    shared_state::ClientMap,
    traits::command_trait::CommandTrait,
    utils::error::BoxError,
    utils::target::{Target, TargetId},
};
use tokio::sync::mpsc;

mod help;
mod info;
mod list;
mod mute;
mod nick;
mod quit;

use help::HelpCommand;
use info::InfoCommand;
use list::ListCommand;
use mute::{MuteCommand, UnmuteCommand};
use nick::NicknameCommand;
use quit::QuitCommand;

/// Enum representing the commands available in the chat system
pub(crate) enum Commands {
    Help,
    Quit,
    Nickname(String),
    List,
    Mute(TargetId),
    Unmute(TargetId),
    Info(Target),
}

impl Commands {
    /// Handles execution of a parsed command from user input.
    /// Returns Ok(true) to continue running, Ok(false) to disconnect.
    pub(crate) async fn handle_command(
        tx: &mpsc::Sender<String>,
        nickname: &mut String,
        input: &str,
        clients: &ClientMap,
        client_id: u32,
    ) -> Result<bool, BoxError> {
        match Self::parse(input) {
            Some(command) => command.execute(tx, nickname, clients, client_id).await,
            None => {
                tx.send(format!(
                    "Unrecognized command: {}. Use /help to see available commands.\n",
                    input
                ))
                .await?;
                Ok(true)
            }
        }
    }

    /// Parses a string of input and returns the corresponding command
    pub fn parse(input: &str) -> Option<Self> {
        let parts: Vec<&str> = input.trim().splitn(2, ' ').collect();
        let command = parts.first()?;

        match *command {
            "/help" => Some(Commands::Help),
            "/quit" => Some(Commands::Quit),
            "/nick" | "/nickname" => {
                parts.get(1).map(|new_nickname| Commands::Nickname(new_nickname.trim().to_string()))
            }
            "/list" => Some(Commands::List),
            "/info" => Target::from_args(parts.get(1).unwrap_or(&"")).map(Commands::Info),
            "/mute" => TargetId::from_args(parts.get(1).unwrap_or(&"")).map(Commands::Mute),
            "/unmute" => TargetId::from_args(parts.get(1).unwrap_or(&"")).map(Commands::Unmute),
            _ => None,
        }
    }

    /// Executes the specific command.
    /// Returns Ok(true) to continue running, Ok(false) to disconnect.
    async fn execute(
        &self,
        tx: &mpsc::Sender<String>,
        nickname: &mut String,
        clients: &ClientMap,
        client_id: u32,
    ) -> Result<bool, BoxError> {
        match self {
            Commands::Help => {
                HelpCommand
                    .execute(tx, nickname, "", clients, client_id)
                    .await?;
                Ok(true)
            }
            Commands::Quit => {
                QuitCommand
                    .execute(tx, nickname, "", clients, client_id)
                    .await?;
                Ok(false) // Signal to disconnect
            }
            Commands::Nickname(new_nickname) => {
                NicknameCommand
                    .execute(tx, nickname, new_nickname, clients, client_id)
                    .await?;
                Ok(true)
            }
            Commands::List => {
                ListCommand
                    .execute(tx, nickname, "", clients, client_id)
                    .await?;
                Ok(true)
            }
            Commands::Mute(target_id) => {
                MuteCommand
                    .execute(tx, nickname, &target_id.0, clients, client_id)
                    .await?;
                Ok(true)
            }
            Commands::Info(target) => {
                InfoCommand
                    .execute(tx, nickname, target.as_str(), clients, client_id)
                    .await?;
                Ok(true)
            }
            Commands::Unmute(target_id) => {
                UnmuteCommand
                    .execute(tx, nickname, &target_id.0, clients, client_id)
                    .await?;
                Ok(true)
            }
        }
    }
}
