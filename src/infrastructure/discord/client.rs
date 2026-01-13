//! Discord API HTTP client.

use async_trait::async_trait;
use reqwest::{header, Client, StatusCode};
use tracing::{debug, warn};

use super::dto::{ErrorResponse, UserResponse};
use crate::domain::entities::{AuthToken, User};
use crate::domain::errors::AuthError;
use crate::domain::ports::AuthPort;

const DISCORD_API_BASE: &str = "https://discord.com/api/v10";
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Discord API authentication client.
pub struct DiscordAuthClient {
    client: Client,
    base_url: String,
}

impl DiscordAuthClient {
    /// Creates new client with default base URL.
    ///
    /// # Errors
    /// Returns error if HTTP client creation fails.
    pub fn new() -> Result<Self, AuthError> {
        Self::with_base_url(DISCORD_API_BASE)
    }

    /// Creates client with custom base URL.
    ///
    /// # Errors
    /// Returns error if HTTP client creation fails.
    pub fn with_base_url(base_url: impl Into<String>) -> Result<Self, AuthError> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| AuthError::unexpected(format!("failed to create HTTP client: {e}")))?;

        Ok(Self {
            client,
            base_url: base_url.into(),
        })
    }

    async fn handle_error_response(
        &self,
        status: StatusCode,
        response: reqwest::Response,
    ) -> AuthError {
        let error_message = match response.json::<ErrorResponse>().await {
            Ok(error) => error.message,
            Err(_) => format!("HTTP {status}"),
        };

        match status {
            StatusCode::UNAUTHORIZED => AuthError::rejected("invalid or expired token"),
            StatusCode::FORBIDDEN => AuthError::rejected(format!("access denied: {error_message}")),
            StatusCode::TOO_MANY_REQUESTS => AuthError::RateLimited { retry_after_ms: 5000 },
            StatusCode::SERVICE_UNAVAILABLE | StatusCode::GATEWAY_TIMEOUT => {
                AuthError::network("Discord API is temporarily unavailable")
            }
            _ => AuthError::unexpected(format!("unexpected response: {status} - {error_message}")),
        }
    }
}

#[async_trait]
impl AuthPort for DiscordAuthClient {
    async fn validate_token(&self, token: &AuthToken) -> Result<User, AuthError> {
        let url = format!("{}/users/@me", self.base_url);

        debug!("Validating token against Discord API");

        let response = self
            .client
            .get(&url)
            .header(header::AUTHORIZATION, token.as_str())
            .send()
            .await
            .map_err(|e| {
                warn!(error = %e, "Failed to connect to Discord API");
                if e.is_timeout() {
                    AuthError::network("request timed out")
                } else if e.is_connect() {
                    AuthError::network("failed to connect to Discord")
                } else {
                    AuthError::network(e.to_string())
                }
            })?;

        let status = response.status();

        if !status.is_success() {
            return Err(self.handle_error_response(status, response).await);
        }

        let user_response: UserResponse = response.json().await.map_err(|e| {
            warn!(error = %e, "Failed to parse user response");
            AuthError::unexpected(format!("failed to parse response: {e}"))
        })?;

        debug!(
            user_id = %user_response.id,
            username = %user_response.username,
            "Token validated successfully"
        );

        Ok(User::new(
            user_response.id,
            user_response.username,
            user_response.discriminator,
            user_response.avatar,
            user_response.bot,
        ))
    }

    async fn health_check(&self) -> Result<(), AuthError> {
        let url = format!("{}/gateway", self.base_url);

        debug!("Performing Discord API health check");

        let response = self.client.get(&url).send().await.map_err(|e| {
            if e.is_timeout() {
                AuthError::network("request timed out")
            } else if e.is_connect() {
                AuthError::network("failed to connect to Discord")
            } else {
                AuthError::network(e.to_string())
            }
        })?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(AuthError::network(format!(
                "Discord API returned {}",
                response.status()
            )))
        }
    }
}

impl Default for DiscordAuthClient {
    fn default() -> Self {
        Self::new().expect("failed to create default Discord client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = DiscordAuthClient::new();
        assert!(client.is_ok());
    }
}
