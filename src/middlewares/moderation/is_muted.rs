use crate::middlewares::MessageContext;
use crate::traits::middleware_trait::{MiddlewareError, MiddlewareTrait};

/// Middleware that checks if a user is muted
pub(crate) struct IsMutedMiddleware;

impl MiddlewareTrait for IsMutedMiddleware {
    fn process<'a>(
        &'a self,
        ctx: &'a mut MessageContext,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), MiddlewareError>> + Send + 'a>>
    {
        Box::pin(async move {
            let clients_lock = ctx.clients.lock().await;

            if let Some(client_state) = clients_lock.get(&ctx.sender_id) {
                if client_state.is_muted() {
                    return Err(MiddlewareError::Blocked(
                        "You are muted and cannot send messages".to_string(),
                    ));
                }
            }

            Ok(())
        })
    }
}
