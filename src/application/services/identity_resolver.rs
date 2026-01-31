use crate::domain::entities::{CachedUser, MessageAuthor, User};
use crate::infrastructure::config::UiConfig;

/// Trait for entities that have identity information.
pub trait Identifiable {
    fn username(&self) -> &str;
    fn discriminator(&self) -> &str;
    fn global_name(&self) -> Option<&str>;
}

impl Identifiable for CachedUser {
    fn username(&self) -> &str {
        self.username()
    }

    fn discriminator(&self) -> &str {
        self.discriminator()
    }

    fn global_name(&self) -> Option<&str> {
        self.global_name()
    }
}

impl Identifiable for User {
    fn username(&self) -> &str {
        self.username()
    }

    fn discriminator(&self) -> &str {
        self.discriminator()
    }

    fn global_name(&self) -> Option<&str> {
        self.global_name()
    }
}

impl Identifiable for MessageAuthor {
    fn username(&self) -> &str {
        self.username()
    }

    fn discriminator(&self) -> &str {
        self.discriminator()
    }

    fn global_name(&self) -> Option<&str> {
        self.global_name.as_deref()
    }
}

/// Resolver for user identity (names) based on preferences.
#[derive(Debug, Clone)]
pub struct IdentityResolver {
    use_display_name: bool,
}

impl IdentityResolver {
    /// Creates a new `IdentityResolver` with the given configuration.
    #[must_use]
    pub fn new(config: &UiConfig) -> Self {
        Self {
            use_display_name: config.use_display_name,
        }
    }

    /// Creates a new `IdentityResolver` with explicit preference.
    #[must_use]
    pub fn with_preference(use_display_name: bool) -> Self {
        Self { use_display_name }
    }

    /// Resolves the name for the user based on the configuration.
    ///
    /// If `use_display_name` is true, it prefers the global name (display name).
    /// Otherwise, it returns the username (with discriminator if legacy).
    #[must_use]
    pub fn resolve(&self, user: &impl Identifiable) -> String {
        if let Some(global_name) = self.use_display_name.then(|| user.global_name()).flatten() {
            return global_name.to_string();
        }

        if user.discriminator() == "0" {
            user.username().to_string()
        } else {
            format!("{}#{}", user.username(), user.discriminator())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::User;

    fn make_user(username: &str, discriminator: &str, global_name: Option<&str>) -> User {
        let mut u = User::new(123_456_789_u64, username, discriminator, None, false, None);
        if let Some(gn) = global_name {
            u = u.with_global_name(gn);
        }
        u
    }

    #[test]
    fn test_prefer_display_name() {
        let user = make_user("username", "0", Some("Global Name"));
        let resolver = IdentityResolver::with_preference(true);
        assert_eq!(resolver.resolve(&user), "Global Name");
    }

    #[test]
    fn test_prefer_username_legacy() {
        let user = make_user("username", "1234", Some("Global Name"));
        let resolver = IdentityResolver::with_preference(false);
        assert_eq!(resolver.resolve(&user), "username#1234");
    }

    #[test]
    fn test_prefer_username_pomelo() {
        let user = make_user("username", "0", Some("Global Name"));
        let resolver = IdentityResolver::with_preference(false);
        assert_eq!(resolver.resolve(&user), "username");
    }

    #[test]
    fn test_fallback_when_no_global_name() {
        let user = make_user("username", "0", None);
        let resolver = IdentityResolver::with_preference(true);
        assert_eq!(resolver.resolve(&user), "username");
    }
}
