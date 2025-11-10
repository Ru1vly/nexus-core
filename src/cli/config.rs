use crate::cli::errors::{CliError, CliResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub user: Option<UserConfig>,
    pub device: Option<DeviceConfig>,
    pub sync: SyncConfig,
    pub network: NetworkConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
    pub auto_migrate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub id: String,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub id: String,
    #[serde(rename = "type")]
    pub device_type: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub enabled: bool,
    pub auto_start: bool,
    pub enable_mdns: bool,
    pub enable_relay: bool,
    pub heartbeat_interval_secs: u64,
    pub max_message_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub listen_port: u16,
    pub listen_address: String,
    pub bootstrap_nodes: Vec<String>,
    pub relay_servers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file: String,
    pub max_size_mb: u64,
    pub max_files: u32,
}

impl Default for Config {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let nexus_dir = home_dir.join(".nexus");
        let db_path = nexus_dir.join("nexus.db");
        let log_path = nexus_dir.join("nexus.log");

        Config {
            database: DatabaseConfig {
                path: db_path.to_string_lossy().to_string(),
                auto_migrate: true,
            },
            user: None,
            device: None,
            sync: SyncConfig {
                enabled: true,
                auto_start: false,
                enable_mdns: true,
                enable_relay: true,
                heartbeat_interval_secs: 10,
                max_message_size: 65536,
            },
            network: NetworkConfig {
                listen_port: 0,
                listen_address: "0.0.0.0".to_string(),
                bootstrap_nodes: vec![],
                relay_servers: vec![],
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "pretty".to_string(),
                file: log_path.to_string_lossy().to_string(),
                max_size_mb: 100,
                max_files: 5,
            },
        }
    }
}

impl Config {
    /// Get the default config path
    pub fn default_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".nexus")
            .join("config.toml")
    }

    /// Get the nexus directory path
    pub fn nexus_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".nexus")
    }

    /// Get the PID file path
    pub fn pid_file() -> PathBuf {
        Self::nexus_dir().join("nexus.pid")
    }

    /// Load configuration from file
    pub fn load(path: Option<&str>) -> CliResult<Self> {
        let config_path = if let Some(p) = path {
            PathBuf::from(p)
        } else {
            Self::default_path()
        };

        if !config_path.exists() {
            return Ok(Config::default());
        }

        let contents = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self, path: Option<&str>) -> CliResult<()> {
        let config_path = if let Some(p) = path {
            PathBuf::from(p)
        } else {
            Self::default_path()
        };

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;
        fs::write(&config_path, contents)?;
        Ok(())
    }

    /// Update a configuration value by key path (e.g., "sync.enabled")
    pub fn set_value(&mut self, key: &str, value: &str) -> CliResult<()> {
        let parts: Vec<&str> = key.split('.').collect();
        if parts.len() != 2 {
            return Err(CliError::ValidationError(
                "Key must be in format: section.key".to_string(),
            ));
        }

        match parts[0] {
            "database" => match parts[1] {
                "path" => self.database.path = value.to_string(),
                "auto_migrate" => self.database.auto_migrate = value.parse().map_err(|_| {
                    CliError::ValidationError("Invalid boolean value".to_string())
                })?,
                _ => return Err(CliError::NotFound(format!("Unknown key: {}", key))),
            },
            "sync" => match parts[1] {
                "enabled" => self.sync.enabled = value.parse().map_err(|_| {
                    CliError::ValidationError("Invalid boolean value".to_string())
                })?,
                "auto_start" => self.sync.auto_start = value.parse().map_err(|_| {
                    CliError::ValidationError("Invalid boolean value".to_string())
                })?,
                "enable_mdns" => self.sync.enable_mdns = value.parse().map_err(|_| {
                    CliError::ValidationError("Invalid boolean value".to_string())
                })?,
                "enable_relay" => self.sync.enable_relay = value.parse().map_err(|_| {
                    CliError::ValidationError("Invalid boolean value".to_string())
                })?,
                "heartbeat_interval_secs" => {
                    self.sync.heartbeat_interval_secs = value.parse().map_err(|_| {
                        CliError::ValidationError("Invalid number value".to_string())
                    })?
                }
                "max_message_size" => {
                    self.sync.max_message_size = value.parse().map_err(|_| {
                        CliError::ValidationError("Invalid number value".to_string())
                    })?
                }
                _ => return Err(CliError::NotFound(format!("Unknown key: {}", key))),
            },
            "network" => match parts[1] {
                "listen_port" => self.network.listen_port = value.parse().map_err(|_| {
                    CliError::ValidationError("Invalid port number".to_string())
                })?,
                "listen_address" => self.network.listen_address = value.to_string(),
                _ => return Err(CliError::NotFound(format!("Unknown key: {}", key))),
            },
            "logging" => match parts[1] {
                "level" => self.logging.level = value.to_string(),
                "format" => self.logging.format = value.to_string(),
                "file" => self.logging.file = value.to_string(),
                "max_size_mb" => self.logging.max_size_mb = value.parse().map_err(|_| {
                    CliError::ValidationError("Invalid number value".to_string())
                })?,
                "max_files" => self.logging.max_files = value.parse().map_err(|_| {
                    CliError::ValidationError("Invalid number value".to_string())
                })?,
                _ => return Err(CliError::NotFound(format!("Unknown key: {}", key))),
            },
            _ => return Err(CliError::NotFound(format!("Unknown section: {}", parts[0]))),
        }

        Ok(())
    }

    /// Get a configuration value by key path
    pub fn get_value(&self, key: &str) -> CliResult<String> {
        let parts: Vec<&str> = key.split('.').collect();
        if parts.len() != 2 {
            return Err(CliError::ValidationError(
                "Key must be in format: section.key".to_string(),
            ));
        }

        let value = match parts[0] {
            "database" => match parts[1] {
                "path" => self.database.path.clone(),
                "auto_migrate" => self.database.auto_migrate.to_string(),
                _ => return Err(CliError::NotFound(format!("Unknown key: {}", key))),
            },
            "sync" => match parts[1] {
                "enabled" => self.sync.enabled.to_string(),
                "auto_start" => self.sync.auto_start.to_string(),
                "enable_mdns" => self.sync.enable_mdns.to_string(),
                "enable_relay" => self.sync.enable_relay.to_string(),
                "heartbeat_interval_secs" => self.sync.heartbeat_interval_secs.to_string(),
                "max_message_size" => self.sync.max_message_size.to_string(),
                _ => return Err(CliError::NotFound(format!("Unknown key: {}", key))),
            },
            "network" => match parts[1] {
                "listen_port" => self.network.listen_port.to_string(),
                "listen_address" => self.network.listen_address.clone(),
                _ => return Err(CliError::NotFound(format!("Unknown key: {}", key))),
            },
            "logging" => match parts[1] {
                "level" => self.logging.level.clone(),
                "format" => self.logging.format.clone(),
                "file" => self.logging.file.clone(),
                "max_size_mb" => self.logging.max_size_mb.to_string(),
                "max_files" => self.logging.max_files.to_string(),
                _ => return Err(CliError::NotFound(format!("Unknown key: {}", key))),
            },
            "user" => match parts[1] {
                "id" => self
                    .user
                    .as_ref()
                    .map(|u| u.id.clone())
                    .ok_or_else(|| CliError::NotFound("User not configured".to_string()))?,
                "name" => self
                    .user
                    .as_ref()
                    .map(|u| u.name.clone())
                    .ok_or_else(|| CliError::NotFound("User not configured".to_string()))?,
                "email" => self
                    .user
                    .as_ref()
                    .map(|u| u.email.clone())
                    .ok_or_else(|| CliError::NotFound("User not configured".to_string()))?,
                _ => return Err(CliError::NotFound(format!("Unknown key: {}", key))),
            },
            "device" => match parts[1] {
                "id" => self
                    .device
                    .as_ref()
                    .map(|d| d.id.clone())
                    .ok_or_else(|| CliError::NotFound("Device not configured".to_string()))?,
                "type" => self
                    .device
                    .as_ref()
                    .map(|d| d.device_type.clone())
                    .ok_or_else(|| CliError::NotFound("Device not configured".to_string()))?,
                "name" => self
                    .device
                    .as_ref()
                    .map(|d| d.name.clone())
                    .ok_or_else(|| CliError::NotFound("Device not configured".to_string()))?,
                _ => return Err(CliError::NotFound(format!("Unknown key: {}", key))),
            },
            _ => return Err(CliError::NotFound(format!("Unknown section: {}", parts[0]))),
        };

        Ok(value)
    }

    /// Expand paths with ~ to full paths
    pub fn expand_path(path: &str) -> String {
        if path.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                return home.join(&path[2..]).to_string_lossy().to_string();
            }
        }
        path.to_string()
    }

    /// Get the expanded database path
    pub fn db_path(&self) -> String {
        Self::expand_path(&self.database.path)
    }

    /// Get the expanded log file path
    pub fn log_path(&self) -> String {
        Self::expand_path(&self.logging.file)
    }
}
