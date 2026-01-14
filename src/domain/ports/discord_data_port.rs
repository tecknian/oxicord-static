//! Discord data port for fetching guilds and channels.

use async_trait::async_trait;

use crate::domain::entities::{AuthToken, Channel, Guild};
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
}
