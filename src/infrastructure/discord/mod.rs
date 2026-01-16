//! Discord API client.

mod client;
mod dto;
pub mod gateway;

pub use client::DiscordClient;
pub use gateway::{
    DispatchEvent, GatewayClient, GatewayClientConfig, GatewayEventKind, GatewayIntents,
    PresenceStatus, TypingIndicatorManager, TypingIndicatorState, TypingUser,
};
