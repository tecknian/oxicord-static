//! Token storage adapters.

#[cfg(feature = "keyring")]
mod keyring_storage;
#[cfg(not(feature = "keyring"))]
mod keyring_storage_stub;

#[cfg(feature = "keyring")]
pub use keyring_storage::KeyringTokenStorage;
#[cfg(not(feature = "keyring"))]
pub use keyring_storage_stub::KeyringTokenStorage;
