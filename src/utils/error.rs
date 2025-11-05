use std::fmt;
use tokio::sync::mpsc;

/// Custom error type for chat operations
#[derive(Debug)]
#[allow(dead_code)]
pub enum ChatError {
    /// Invalid user ID format (not a valid u32)
    InvalidUserId(String),
    /// User not found in the client map
    UserNotFound(String),
    /// Nickname validation errors
    NicknameEmpty,
    NicknameTooLong {
        max: usize,
    },
    NicknameInvalid(String),
    NicknameAlreadyTaken(String),
    /// Message sending failed
    MessageSendFailed,
    /// Generic validation failure
    ValidationFailed(String),
    /// User is muted
    Muted,
    /// Target cannot be empty
    TargetEmpty,
}

impl fmt::Display for ChatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChatError::InvalidUserId(id) => write!(f, "Invalid user ID: {}", id),
            ChatError::UserNotFound(target) => write!(f, "User '{}' not found", target),
            ChatError::NicknameEmpty => write!(f, "Nickname cannot be empty"),
            ChatError::NicknameTooLong { max } => {
                write!(f, "Nickname too long (max {} chars)", max)
            }
            ChatError::NicknameInvalid(reason) => write!(f, "Invalid nickname: {}", reason),
            ChatError::NicknameAlreadyTaken(nickname) => {
                write!(f, "Nickname '{}' is already in use", nickname)
            }
            ChatError::MessageSendFailed => write!(f, "Failed to send message"),
            ChatError::ValidationFailed(reason) => write!(f, "Validation failed: {}", reason),
            ChatError::Muted => write!(f, "You are muted and cannot send messages"),
            ChatError::TargetEmpty => write!(f, "Target cannot be empty"),
        }
    }
}

impl std::error::Error for ChatError {}

impl From<mpsc::error::SendError<String>> for ChatError {
    fn from(_: mpsc::error::SendError<String>) -> Self {
        ChatError::MessageSendFailed
    }
}

// Type aliases for convenience
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
#[allow(dead_code)]
pub type ChatResult<T> = Result<T, ChatError>;
