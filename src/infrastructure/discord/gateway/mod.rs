mod client;
mod codec;
mod connection;
mod constants;
mod error;
mod events;
mod heartbeat;
mod payloads;
mod session;
mod state;
mod typing;

pub use client::{GatewayClient, GatewayClientConfig};
pub use connection::GatewayConnection;
pub use constants::{GatewayIntent, GatewayIntents, GatewayOpcode};
pub use error::{GatewayCloseCode, GatewayError, GatewayResult};
pub use events::{
    Activity, ActivityKind, DispatchEvent, GatewayEventKind, PresenceStatus, ReactionEmoji,
    TypingUser, UnavailableGuild,
};
pub use session::SessionInfo;
pub use state::{ConnectionState, GatewayState};
pub use typing::{TypingIndicatorManager, TypingIndicatorState};
