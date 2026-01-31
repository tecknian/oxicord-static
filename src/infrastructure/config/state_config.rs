use serde::{Deserialize, Serialize};

/// Session state configuration.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StateConfig {
    /// Last accessed channel ID.
    #[serde(default)]
    pub last_channel_id: Option<String>,

    /// Last accessed guild ID.
    #[serde(default)]
    pub last_guild_id: Option<String>,
}
