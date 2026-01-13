//! Discord user entity.

use serde::{Deserialize, Serialize};

/// Discord user information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    id: String,
    username: String,
    discriminator: String,
    avatar: Option<String>,
    bot: bool,
}

impl User {
    /// Creates new user.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        username: impl Into<String>,
        discriminator: impl Into<String>,
        avatar: Option<String>,
        bot: bool,
    ) -> Self {
        Self {
            id: id.into(),
            username: username.into(),
            discriminator: discriminator.into(),
            avatar,
            bot,
        }
    }

    /// Returns user ID.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns username.
    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns discriminator.
    #[must_use]
    pub fn discriminator(&self) -> &str {
        &self.discriminator
    }

    /// Returns avatar hash.
    #[must_use]
    pub fn avatar(&self) -> Option<&str> {
        self.avatar.as_deref()
    }

    /// Returns whether user is a bot.
    #[must_use]
    pub fn is_bot(&self) -> bool {
        self.bot
    }

    /// Returns display name.
    #[must_use]
    pub fn display_name(&self) -> String {
        if self.discriminator == "0" {
            self.username.clone()
        } else {
            format!("{}#{}", self.username, self.discriminator)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new(
            "123456789",
            "testuser",
            "1234",
            Some("abc123".into()),
            false,
        );

        assert_eq!(user.id(), "123456789");
        assert_eq!(user.username(), "testuser");
        assert_eq!(user.discriminator(), "1234");
        assert_eq!(user.avatar(), Some("abc123"));
        assert!(!user.is_bot());
    }

    #[test]
    fn test_display_name_legacy() {
        let user = User::new("123", "olduser", "1234", None, false);
        assert_eq!(user.display_name(), "olduser#1234");
    }

    #[test]
    fn test_display_name_new_style() {
        let user = User::new("123", "newuser", "0", None, false);
        assert_eq!(user.display_name(), "newuser");
    }
}
