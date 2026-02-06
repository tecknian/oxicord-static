use serde::{Deserialize, Serialize};

use super::Permissions;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RoleId(#[serde(with = "crate::domain::serde_utils::string_to_u64")] pub u64);

impl RoleId {
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for RoleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for RoleId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<&str> for RoleId {
    fn from(value: &str) -> Self {
        Self(value.parse().unwrap_or(0))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Role {
    pub id: RoleId,
    pub name: String,
    #[serde(default)]
    pub color: u32,
    #[serde(default)]
    pub hoist: bool,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub unicode_emoji: Option<String>,
    #[serde(default)]
    pub position: i32,
    pub permissions: Permissions,
    #[serde(default)]
    pub managed: bool,
    #[serde(default)]
    pub mentionable: bool,
}

impl Role {
    #[must_use]
    pub const fn id(&self) -> RoleId {
        self.id
    }
}
