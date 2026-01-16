//! Token resolution use case.

use std::sync::Arc;

use tracing::{debug, info};

use crate::application::dto::TokenSource;
use crate::domain::entities::AuthToken;
use crate::domain::errors::AuthError;
use crate::domain::ports::TokenStoragePort;

/// Environment variable for Discord token.
pub const TOKEN_ENV_VAR: &str = "DISCORDO_TOKEN";

/// Resolved token with its source.
#[derive(Debug, Clone)]
pub struct ResolvedToken {
    /// The authentication token.
    pub token: AuthToken,
    /// Source of the token.
    pub source: TokenSource,
}

impl ResolvedToken {
    /// Creates new resolved token.
    #[must_use]
    pub const fn new(token: AuthToken, source: TokenSource) -> Self {
        Self { token, source }
    }
}

/// Resolves authentication token from available sources.
pub struct ResolveTokenUseCase {
    storage_port: Arc<dyn TokenStoragePort>,
}

impl ResolveTokenUseCase {
    /// Creates new use case.
    #[must_use]
    pub const fn new(storage_port: Arc<dyn TokenStoragePort>) -> Self {
        Self { storage_port }
    }

    /// Resolves token from CLI, env, or keyring.
    ///
    /// # Errors
    /// Returns error if storage access fails.
    pub async fn execute(
        &self,
        cli_token: Option<String>,
    ) -> Result<Option<ResolvedToken>, AuthError> {
        if let Some(token_str) = cli_token.filter(|s| !s.trim().is_empty()) {
            debug!("Checking command-line token");
            if let Some(token) = AuthToken::new(&token_str) {
                info!("Using token from command line");
                return Ok(Some(ResolvedToken::new(token, TokenSource::CommandLine)));
            }
            debug!("Command-line token has invalid format, trying other sources");
        }

        if let Ok(token_str) = std::env::var(TOKEN_ENV_VAR) {
            let token_str = token_str.trim().to_string();
            if !token_str.is_empty() {
                debug!("Checking environment variable token");
                if let Some(token) = AuthToken::new(&token_str) {
                    info!("Using token from environment variable");
                    return Ok(Some(ResolvedToken::new(token, TokenSource::Environment)));
                }
                debug!("Environment token has invalid format, trying keyring");
            }
        }

        debug!("Checking keyring for stored token");
        match self.storage_port.get_token().await {
            Ok(Some(token)) => {
                info!("Using token from system keyring");
                Ok(Some(ResolvedToken::new(token, TokenSource::Keyring)))
            }
            Ok(None) => {
                debug!("No token found in any source");
                Ok(None)
            }
            Err(e) => {
                debug!(error = %e, "Failed to check keyring, treating as no token");
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ports::mocks::MockTokenStorage;
    use std::env;

    fn make_valid_token() -> String {
        "MTIzNDU2Nzg5MDEyMzQ1Njc4OQ.XXXXXX.YYYYYYYYYYYYYYYYYYYYYYYYYYYY".to_string()
    }

    #[tokio::test]
    async fn test_cli_token_priority() {
        let storage = Arc::new(MockTokenStorage::with_token(AuthToken::new_unchecked(
            "keyring.token.here",
        )));
        let use_case = ResolveTokenUseCase::new(storage);

        let result = use_case.execute(Some(make_valid_token())).await.unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().source, TokenSource::CommandLine);
    }

    #[tokio::test]
    async fn test_keyring_fallback() {
        let storage = Arc::new(MockTokenStorage::with_token(AuthToken::new_unchecked(
            make_valid_token(),
        )));
        let use_case = ResolveTokenUseCase::new(storage);

        unsafe { env::remove_var(TOKEN_ENV_VAR) };

        let result = use_case.execute(None).await.unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().source, TokenSource::Keyring);
    }

    #[tokio::test]
    async fn test_no_token_found() {
        let storage = Arc::new(MockTokenStorage::new());
        let use_case = ResolveTokenUseCase::new(storage);

        unsafe { env::remove_var(TOKEN_ENV_VAR) };

        let result = use_case.execute(None).await.unwrap();

        assert!(result.is_none());
    }
}
