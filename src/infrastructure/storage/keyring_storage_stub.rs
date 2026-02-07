//! Stub keyring storage for builds without keyring support.

use async_trait::async_trait;
use tracing::debug;

use crate::domain::entities::AuthToken;
use crate::domain::errors::AuthError;
use crate::domain::ports::TokenStoragePort;

/// Stub token storage that does nothing.
/// Used when keyring feature is disabled.
pub struct KeyringTokenStorage;

impl KeyringTokenStorage {
    /// Creates new stub storage.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Creates storage with custom names (no-op in stub).
    #[must_use]
    pub fn with_names(_service: impl Into<String>, _user: impl Into<String>) -> Self {
        Self
    }
}

impl Default for KeyringTokenStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TokenStoragePort for KeyringTokenStorage {
    async fn get_token(&self) -> Result<Option<AuthToken>, AuthError> {
        debug!("Keyring feature disabled - no token storage available");
        Ok(None)
    }

    async fn store_token(&self, _token: &AuthToken) -> Result<(), AuthError> {
        debug!("Keyring feature disabled - cannot store token");
        Ok(())
    }

    async fn delete_token(&self) -> Result<(), AuthError> {
        debug!("Keyring feature disabled - cannot delete token");
        Ok(())
    }
}
