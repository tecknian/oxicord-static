use serde::{Deserialize, Serialize};

use super::{RoleId, User, UserId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Member {
    pub user: Option<User>,
    pub nick: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub roles: Vec<RoleId>,
    pub joined_at: String,
    pub premium_since: Option<String>,
    #[serde(default)]
    pub deaf: bool,
    #[serde(default)]
    pub mute: bool,
    #[serde(default)]
    pub pending: bool,
    pub permissions: Option<String>, // Calculated permissions (sometimes sent)
    pub communication_disabled_until: Option<String>,
}

impl Member {
    #[must_use]
    pub fn user_id(&self) -> Option<UserId> {
        self.user.as_ref().map(User::id)
    }

    #[must_use]
    pub fn roles(&self) -> &[RoleId] {
        &self.roles
    }
}
