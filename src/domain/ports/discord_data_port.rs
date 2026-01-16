//! Discord data port for fetching guilds and channels.

use async_trait::async_trait;

use crate::domain::entities::{AuthToken, Channel, Guild, Message};
use crate::domain::errors::AuthError;

/// Represents a direct message channel with a recipient.
#[derive(Debug, Clone)]
pub struct DirectMessageChannel {
    /// The unique channel ID for this DM conversation.
    pub channel_id: String,
    /// The recipient user's ID.
    pub recipient_id: String,
    /// The recipient's display name.
    pub recipient_name: String,
}

/// Options for fetching messages from a channel.
#[derive(Debug, Clone, Default)]
pub struct FetchMessagesOptions {
    pub limit: Option<u8>,
    pub before: Option<u64>,
    pub after: Option<u64>,
    pub around: Option<u64>,
}

impl FetchMessagesOptions {
    #[must_use]
    pub const fn with_limit(mut self, limit: u8) -> Self {
        self.limit = Some(if limit < 100 { limit } else { 100 });
        self
    }

    #[must_use]
    pub const fn before_message(mut self, message_id: u64) -> Self {
        self.before = Some(message_id);
        self
    }

    #[must_use]
    pub const fn after_message(mut self, message_id: u64) -> Self {
        self.after = Some(message_id);
        self
    }
}

/// Port for fetching Discord data (guilds, channels, DMs, etc).
#[async_trait]
pub trait DiscordDataPort: Send + Sync {
    /// Fetches all guilds the user is a member of.
    async fn fetch_guilds(&self, token: &AuthToken) -> Result<Vec<Guild>, AuthError>;

    /// Fetches all channels for a given guild.
    async fn fetch_channels(
        &self,
        token: &AuthToken,
        guild_id: u64,
    ) -> Result<Vec<Channel>, AuthError>;

    /// Fetches all direct message channels for the user.
    async fn fetch_dm_channels(
        &self,
        token: &AuthToken,
    ) -> Result<Vec<DirectMessageChannel>, AuthError>;

    /// Fetches messages from a channel.
    async fn fetch_messages(
        &self,
        token: &AuthToken,
        channel_id: u64,
        options: FetchMessagesOptions,
    ) -> Result<Vec<Message>, AuthError>;
}
