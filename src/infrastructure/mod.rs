//! Infrastructure layer with external service adapters.

/// Application configuration.
pub mod config;
/// Discord API client.
pub mod discord;
/// Token storage adapters.
pub mod storage;

pub use config::{AppConfig, LogLevel};
pub use discord::{
    DiscordClient, DispatchEvent, GatewayClient, GatewayClientConfig, GatewayCommand,
    GatewayEventKind, GatewayIntents, PresenceStatus, TypingIndicatorManager, TypingIndicatorState,
    TypingUser,
};
pub use storage::KeyringTokenStorage;
