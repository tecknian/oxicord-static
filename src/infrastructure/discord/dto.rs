//! Discord API response DTOs.

use serde::Deserialize;

/// Discord API user response structure.
#[derive(Debug, Deserialize)]
pub struct UserResponse {
    /// Discord user ID.
    pub id: String,
    /// Discord username.
    pub username: String,
    /// User discriminator tag.
    pub discriminator: String,
    /// Optional avatar hash.
    pub avatar: Option<String>,
    /// Whether the user is a bot.
    #[serde(default)]
    pub bot: bool,
}

/// Discord API error response structure.
#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    /// Error message from Discord.
    pub message: String,
}

/// Discord API guild response structure from /users/@me/guilds.
#[derive(Debug, Deserialize)]
pub struct GuildResponse {
    /// Guild ID (snowflake as string).
    pub id: String,
    /// Guild name.
    #[serde(default)]
    pub name: String,
    /// Guild icon hash (null if no custom icon).
    pub icon: Option<String>,
    /// Whether the current user owns this guild.
    #[serde(default)]
    #[allow(dead_code)]
    pub owner: bool,
    /// User's permissions in this guild (string of permission bits).
    #[serde(default)]
    #[allow(dead_code)]
    pub permissions: Option<String>,
    /// Guild feature flags.
    #[serde(default)]
    #[allow(dead_code)]
    pub features: Vec<String>,
}

/// Discord API channel response structure.
#[derive(Debug, Deserialize)]
pub struct ChannelResponse {
    /// Channel ID.
    pub id: String,
    /// Channel type.
    #[serde(rename = "type")]
    pub kind: u8,
    /// Guild ID (if guild channel).
    #[allow(dead_code)]
    pub guild_id: Option<String>,
    /// Channel name.
    pub name: Option<String>,
    /// Parent category ID.
    pub parent_id: Option<String>,
    /// Position in channel list.
    #[serde(default)]
    pub position: i32,
    /// Channel topic.
    pub topic: Option<String>,
}

/// Discord API DM recipient structure.
#[derive(Debug, Deserialize)]
pub struct DmRecipient {
    /// User ID.
    pub id: String,
    /// Username.
    pub username: String,
    /// Global display name.
    #[serde(default)]
    pub global_name: Option<String>,
}

/// Discord API DM channel response structure.
#[derive(Debug, Deserialize)]
pub struct DmChannelResponse {
    /// Channel ID.
    pub id: String,
    /// Channel type (DM = 1, Group DM = 3).
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub kind: u8,
    /// List of recipients in this DM.
    #[serde(default)]
    pub recipients: Vec<DmRecipient>,
}
