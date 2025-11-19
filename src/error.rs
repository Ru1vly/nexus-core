use std::fmt;

/// Main error type for the ahenk library
#[derive(Debug)]
pub enum AhenkError {
    /// Database-related errors
    Database(rusqlite::Error),
    /// Validation errors (e.g., empty fields, invalid input)
    Validation(String),
    /// Authentication/authorization errors
    Auth(String),
    /// Resource not found errors
    NotFound(String),
    /// Serialization/deserialization errors
    Serialization(String),
    /// P2P synchronization errors
    Sync(String),
    /// I/O errors
    Io(std::io::Error),
    /// Generic errors with custom messages
    Other(String),
}

impl fmt::Display for AhenkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AhenkError::Database(e) => write!(f, "Database error: {}", e),
            AhenkError::Validation(msg) => write!(f, "Validation error: {}", msg),
            AhenkError::Auth(msg) => write!(f, "Authentication error: {}", msg),
            AhenkError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AhenkError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            AhenkError::Sync(msg) => write!(f, "Synchronization error: {}", msg),
            AhenkError::Io(e) => write!(f, "I/O error: {}", e),
            AhenkError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for AhenkError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AhenkError::Database(e) => Some(e),
            AhenkError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for AhenkError {
    fn from(err: rusqlite::Error) -> Self {
        AhenkError::Database(err)
    }
}

impl From<std::io::Error> for AhenkError {
    fn from(err: std::io::Error) -> Self {
        AhenkError::Io(err)
    }
}

impl From<String> for AhenkError {
    fn from(msg: String) -> Self {
        AhenkError::Other(msg)
    }
}

impl From<&str> for AhenkError {
    fn from(msg: &str) -> Self {
        AhenkError::Other(msg.to_string())
    }
}

/// Result type alias for ahenk operations
pub type Result<T> = std::result::Result<T, AhenkError>;
