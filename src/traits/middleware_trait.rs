use crate::middlewares::MessageContext;
use std::future::Future;
use std::pin::Pin;

/// Trait that all middleware must implement
///
/// Middleware process messages before they are broadcast to other clients.
/// They can block, modify, or validate messages.
pub(crate) trait MiddlewareTrait: Send + Sync {
    /// Process a message through this middleware
    fn process<'a>(
        &'a self,
        ctx: &'a mut MessageContext,
    ) -> Pin<Box<dyn Future<Output = Result<(), MiddlewareError>> + Send + 'a>>;
}

/// Errors that can be returned by middleware
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum MiddlewareError {
    /// Message is blocked with a reason
    Blocked(String),
    /// Message failed validation
    ValidationFailed(String),
}

impl std::fmt::Display for MiddlewareError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MiddlewareError::Blocked(msg) => write!(f, "{}", msg),
            MiddlewareError::ValidationFailed(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for MiddlewareError {}
