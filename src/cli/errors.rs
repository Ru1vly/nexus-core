use std::fmt;

pub type CliResult<T> = Result<T, CliError>;

#[derive(Debug)]
pub enum CliError {
    ConfigError(String),
    DatabaseError(String),
    IoError(std::io::Error),
    SyncError(String),
    DaemonError(String),
    ValidationError(String),
    AuthError(String),
    NotFound(String),
    Other(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            CliError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            CliError::IoError(err) => write!(f, "IO error: {}", err),
            CliError::SyncError(msg) => write!(f, "Sync error: {}", msg),
            CliError::DaemonError(msg) => write!(f, "Daemon error: {}", msg),
            CliError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            CliError::AuthError(msg) => write!(f, "Authentication error: {}", msg),
            CliError::NotFound(msg) => write!(f, "Not found: {}", msg),
            CliError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for CliError {}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        CliError::IoError(err)
    }
}

impl From<toml::de::Error> for CliError {
    fn from(err: toml::de::Error) -> Self {
        CliError::ConfigError(err.to_string())
    }
}

impl From<toml::ser::Error> for CliError {
    fn from(err: toml::ser::Error) -> Self {
        CliError::ConfigError(err.to_string())
    }
}

impl From<String> for CliError {
    fn from(err: String) -> Self {
        CliError::Other(err)
    }
}

impl From<&str> for CliError {
    fn from(err: &str) -> Self {
        CliError::Other(err.to_string())
    }
}

impl From<rusqlite::Error> for CliError {
    fn from(err: rusqlite::Error) -> Self {
        CliError::DatabaseError(err.to_string())
    }
}
