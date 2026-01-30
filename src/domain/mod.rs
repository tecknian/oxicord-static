//! Domain layer with core business entities and port definitions.

/// Connection status definitions.
pub mod connection;
/// Entity definitions.
pub mod entities;
/// Error types.
pub mod errors;
/// Keybinding definitions.
pub mod keybinding;
/// Port definitions.
pub mod ports;
/// Serde utilities.
pub mod serde_utils;

pub use connection::ConnectionStatus;
pub use entities::{AuthToken, User};
pub use errors::AuthError;
pub use ports::{AuthPort, TokenStoragePort};
