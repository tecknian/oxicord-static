//! Token resolution use case.

use std::sync::Arc;

use tracing::{debug, info};

use crate::application::dto::TokenSource;
use crate::domain::entities::AuthToken;
use crate::domain::errors::AuthError;
use crate::domain::ports::TokenStoragePort;

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

    /// Resolves token from keyring or CLI/Env.
    ///
    /// Priority:
    /// 1. Keyring
    /// 2. CLI/Env (passed as argument)
    ///
    /// # Errors
    /// Returns error if storage access fails.
    pub async fn execute(
        &self,
        cli_token: Option<String>,
    ) -> Result<Option<ResolvedToken>, AuthError> {
        debug!("Checking keyring for stored token");
        match self.storage_port.get_token().await {
            Ok(Some(token)) => {
                info!("Using token from system keyring");
                return Ok(Some(ResolvedToken::new(token, TokenSource::Keyring)));
            }
            Ok(None) => {
                debug!("No token found in keyring");
            }
            Err(e) => {
                debug!(error = %e, "Failed to check keyring");
            }
        }

        if let Some(token_str) = cli_token.filter(|s| !s.trim().is_empty()) {
            debug!("Checking command-line/env token");
            if let Some(token) = AuthToken::new(&token_str) {
                info!("Using token from command line / environment");
                return Ok(Some(ResolvedToken::new(token, TokenSource::CommandLine)));
            }
            debug!("Command-line token has invalid format");
        }

        debug!("No token found in any source");
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ports::mocks::MockTokenStorage;

    fn make_valid_token() -> String {
        "MTIzNDU2Nzg5MDEyMzQ1Njc4OQ.XXXXXX.YYYYYYYYYYYYYYYYYYYYYYYYYYYY".to_string()
    }

    #[tokio::test]
    async fn test_keyring_priority() {
        let storage = Arc::new(MockTokenStorage::with_token(AuthToken::new_unchecked(
            make_valid_token(),
        )));
        let use_case = ResolveTokenUseCase::new(storage);

        let result = use_case
            .execute(Some("cli.token.here".to_string()))
            .await
            .unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().source, TokenSource::Keyring);
    }

    #[tokio::test]
    async fn test_cli_fallback() {
        let storage = Arc::new(MockTokenStorage::new());
        let use_case = ResolveTokenUseCase::new(storage);

        let result = use_case.execute(Some(make_valid_token())).await.unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().source, TokenSource::CommandLine);
    }

    #[tokio::test]
    async fn test_no_token_found() {
        let storage = Arc::new(MockTokenStorage::new());
        let use_case = ResolveTokenUseCase::new(storage);

        let result = use_case.execute(None).await.unwrap();

        assert!(result.is_none());
    }
}
