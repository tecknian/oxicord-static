//! Port definitions for external adapters.

mod auth_port;
mod token_storage_port;

pub use auth_port::AuthPort;
pub use token_storage_port::TokenStoragePort;

#[cfg(test)]
/// Mock implementations for testing.
pub mod mocks {
    pub use super::auth_port::mock::MockAuthPort;
    pub use super::token_storage_port::mock::MockTokenStorage;
}
