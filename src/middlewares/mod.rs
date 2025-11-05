use crate::shared_state::ClientMap;
use crate::traits::middleware_trait::{MiddlewareError, MiddlewareTrait};

pub(crate) mod moderation;

/// Context passed through the middleware chain
pub(crate) struct MessageContext {
    pub message: String,
    pub sender_id: u32,
    #[allow(dead_code)]
    pub nickname: String,
    pub clients: ClientMap,
}

/// Chain of middleware that processes messages sequentially
pub(crate) struct MiddlewareChain {
    middlewares: Vec<Box<dyn MiddlewareTrait>>,
}

impl MiddlewareChain {
    pub fn new() -> Self {
        Self {
            middlewares: vec![],
        }
    }

    pub fn add(mut self, middleware: Box<dyn MiddlewareTrait>) -> Self {
        self.middlewares.push(middleware);
        self
    }

    /// Process a message through all middleware in the chain
    pub async fn process(&self, ctx: &mut MessageContext) -> Result<(), MiddlewareError> {
        for middleware in &self.middlewares {
            middleware.process(ctx).await?;
        }
        Ok(())
    }
}

impl Default for MiddlewareChain {
    fn default() -> Self {
        Self::new().add(Box::new(moderation::IsMutedMiddleware))
    }
}
