//! Infrastructure layer with external service adapters.

/// Application configuration.
pub mod config;
/// Discord API client.
pub mod discord;
/// Image handling (caching, loading, CDN optimization).
pub mod image;
/// Token storage adapters.
pub mod storage;

pub use config::{AppConfig, LogLevel};
pub use discord::{
    DiscordClient, DispatchEvent, GatewayClient, GatewayClientConfig, GatewayCommand,
    GatewayEventKind, GatewayIntents, PresenceStatus, TypingIndicatorManager, TypingIndicatorState,
    TypingUser,
};
pub use image::{
    CacheStats, DiskImageCache, ImageLoadedEvent, ImageLoader, ImageLoaderConfig, MemoryImageCache,
    extract_attachment_id, is_discord_cdn_url, optimize_cdn_url, optimize_cdn_url_default,
};
pub use storage::KeyringTokenStorage;
