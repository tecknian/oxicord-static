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
