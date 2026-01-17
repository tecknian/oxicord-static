//! Domain layer with core business entities and port definitions.

/// Entity definitions.
pub mod entities;
/// Error types.
pub mod errors;
/// Keybinding definitions.
pub mod keybinding;
/// Port definitions.
pub mod ports;

pub use entities::{AuthToken, User};
pub use errors::AuthError;
pub use ports::{AuthPort, TokenStoragePort};
