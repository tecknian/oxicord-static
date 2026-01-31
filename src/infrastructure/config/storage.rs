use super::app_config::AppConfig;
use super::state_config::StateConfig;
use directories::ProjectDirs;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{info, warn};

const APP_QUALIFIER: &str = "com";
const APP_ORGANIZATION: &str = "linuxmobile";
const APP_NAME: &str = "oxicord";
const CONFIG_FILE_NAME: &str = "config.toml";
const STATE_FILE_NAME: &str = "state.toml";

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to determine config directory")]
    ConfigDirNotFound,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),
    #[error("toml deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),
}

pub struct StorageManager {
    config_dir: PathBuf,
}

impl StorageManager {
    /// Create a new `StorageManager`.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if the configuration directory cannot be determined.
    pub fn new() -> Result<Self, ConfigError> {
        let config_dir = ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_NAME)
            .map(|dirs| dirs.config_dir().to_path_buf())
            .ok_or(ConfigError::ConfigDirNotFound)?;

        Ok(Self { config_dir })
    }

    /// Creates a new `StorageManager` with a specific directory (useful for testing).
    #[must_use]
    pub fn with_dir(path: PathBuf) -> Self {
        Self { config_dir: path }
    }

    /// Returns the configuration directory path.
    #[must_use]
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Ensures the configuration directory exists.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if the directory cannot be created.
    pub fn ensure_config_dir(&self) -> Result<(), ConfigError> {
        if !self.config_dir.exists() {
            info!("Creating configuration directory at {:?}", self.config_dir);
            fs::create_dir_all(&self.config_dir)?;
        }
        Ok(())
    }

    /// Loads the application configuration.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if the file cannot be read or parsed.
    pub fn load_config(&self, path_override: Option<&Path>) -> Result<AppConfig, ConfigError> {
        self.ensure_config_dir()?;
        let config_path = path_override.map_or_else(
            || self.config_dir.join(CONFIG_FILE_NAME),
            std::path::Path::to_path_buf,
        );

        if !config_path.exists() {
            info!(
                "Config file not found at {:?}, creating default.",
                config_path
            );
            let default_config = AppConfig::default();
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)?;
            }
            Self::save_to_file(&config_path, &default_config)?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&config_path)?;
        match toml::from_str::<AppConfig>(&content) {
            Ok(config) => Ok(config),
            Err(e) => {
                warn!("Failed to parse config file: {}. Using defaults.", e);
                Ok(AppConfig::default())
            }
        }
    }

    /// Loads the application state.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if the file cannot be read or parsed.
    pub fn load_state(&self) -> Result<StateConfig, ConfigError> {
        self.ensure_config_dir()?;
        let state_path = self.config_dir.join(STATE_FILE_NAME);

        if !state_path.exists() {
            return Ok(StateConfig::default());
        }

        let content = fs::read_to_string(&state_path)?;
        match toml::from_str::<StateConfig>(&content) {
            Ok(state) => Ok(state),
            Err(e) => {
                warn!("Failed to parse state file: {}. Resetting state.", e);
                Ok(StateConfig::default())
            }
        }
    }

    /// Saves the application state.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if the file cannot be written.
    pub fn save_state(&self, state: &StateConfig) -> Result<(), ConfigError> {
        self.ensure_config_dir()?;
        let state_path = self.config_dir.join(STATE_FILE_NAME);
        Self::save_to_file(&state_path, state)
    }

    fn save_to_file<T: serde::Serialize>(path: &Path, data: &T) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(data)?;

        let parent = path
            .parent()
            .ok_or_else(|| std::io::Error::other("Invalid path"))?;
        let mut temp_file = tempfile::NamedTempFile::new_in(parent)?;
        temp_file.write_all(content.as_bytes())?;
        temp_file.persist(path).map_err(|e| e.error)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_ensure_config_dir_creates_directory() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("oxicord");
        let manager = StorageManager::with_dir(config_path.clone());

        assert!(!config_path.exists());
        manager.ensure_config_dir().unwrap();
        assert!(config_path.exists());
    }

    #[test]
    fn test_load_config_creates_default_if_missing() {
        let dir = tempdir().unwrap();
        let manager = StorageManager::with_dir(dir.path().to_path_buf());

        let config = manager.load_config(None).unwrap();
        assert_eq!(config.mouse, true); // Default value

        let config_file = dir.path().join(CONFIG_FILE_NAME);
        assert!(config_file.exists());
    }

    #[test]
    fn test_load_config_handles_malformed_file() {
        let dir = tempdir().unwrap();
        let manager = StorageManager::with_dir(dir.path().to_path_buf());
        let config_file = dir.path().join(CONFIG_FILE_NAME);

        fs::write(&config_file, "invalid_toml = [").unwrap();

        let config = manager.load_config(None).unwrap();
        assert_eq!(config.mouse, true);
        let content = fs::read_to_string(&config_file).unwrap();
        assert_eq!(content, "invalid_toml = [");
    }

    #[test]
    fn test_save_and_load_state() {
        let dir = tempdir().unwrap();
        let manager = StorageManager::with_dir(dir.path().to_path_buf());

        let mut state = StateConfig::default();
        state.last_channel_id = Some("123".to_string());

        manager.save_state(&state).unwrap();

        let loaded_state = manager.load_state().unwrap();
        assert_eq!(loaded_state.last_channel_id, Some("123".to_string()));
    }

    #[test]
    fn test_atomic_write() {
        let dir = tempdir().unwrap();
        let manager = StorageManager::with_dir(dir.path().to_path_buf());
        let state_path = dir.path().join(STATE_FILE_NAME);

        let state = StateConfig::default();
        manager.save_state(&state).unwrap();

        assert!(state_path.exists());
    }
}
