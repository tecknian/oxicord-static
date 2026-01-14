use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::domain::entities::{AuthToken, ChannelId, Message, MessageId};
use crate::domain::errors::AuthError;

#[derive(Debug, Clone)]
pub enum GatewayEvent {
    Ready {
        session_id: String,
    },
    MessageCreate {
        message: Message,
    },
    MessageUpdate {
        message: Message,
    },
    MessageDelete {
        message_id: MessageId,
        channel_id: ChannelId,
    },
    Reconnecting,
    Disconnected,
    Error {
        message: String,
    },
}

#[async_trait]
pub trait GatewayPort: Send + Sync {
    async fn connect(
        &self,
        token: &AuthToken,
    ) -> Result<mpsc::UnboundedReceiver<GatewayEvent>, AuthError>;

    async fn disconnect(&self) -> Result<(), AuthError>;

    fn is_connected(&self) -> bool;
}
