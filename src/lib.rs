//! # Ahenk - Conflict-Free Offline Synchronization Tool
//!
//! Cross-platform database synchronization infrastructure with P2P networking
//! and CRDT-based conflict resolution.
//!
//! ## Overview
//!
//! Ahenk provides a complete solution for synchronizing databases across devices:
//! - **User authentication** with Argon2 password hashing
//! - **Device management** and authorization
//! - **P2P networking** using libp2p (mDNS, relay, DCUtR)
//! - **CRDT synchronization** with hybrid logical clocks
//! - **Operation log** for tracking all changes
//! - **Offline-first** architecture with conflict resolution
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use ahenk::{initialize_database, register_user, login_user, add_device_to_user};
//!
//! // Initialize database with automatic migrations
//! let conn = initialize_database("app.db").unwrap();
//!
//! // Register a new user
//! let user = register_user(
//!     &conn,
//!     "alice".to_string(),
//!     "alice@example.com".to_string(),
//!     "secure_password".to_string(),
//! ).unwrap();
//!
//! // Add a device
//! let device = add_device_to_user(&conn, user.user_id, "ios".to_string(), None).unwrap();
//! ```
//!
//! ## Modules
//!
//! - `models`: Core data structures (User, Device, OplogEntry, Peer)
//! - `db`: Database operations and migrations
//! - `logic`: Business logic (user management, device management, sync)
//! - `crdt`: CRDT implementation with hybrid logical clocks
//! - `auth`: Device authorization workflows
//! - `error`: Error types and result aliases

// Internal modules
pub mod auth;
pub mod crdt;
pub mod db;
pub mod error;
pub mod ffi;
pub mod logic;
pub mod models;
pub mod tauri_api;

// CLI module (optional, enabled with "cli" feature)
#[cfg(feature = "cli")]
pub mod cli;

// ============================================================================
// Error Types
// ============================================================================

pub use error::{AhenkError, Result};

// ============================================================================
// Core Models
// ============================================================================

pub use models::{Device, OplogEntry, Peer, User};

// ============================================================================
// Database Operations
// ============================================================================

// Initialization and migrations
pub use db::migrations::{apply_migrations, get_current_version, get_migration_history};
pub use db::operations::initialize_database;

// User operations
pub use db::operations::{create_user, get_user, get_user_by_mail, get_user_by_name};

// Device operations
pub use db::operations::{
    create_device, get_device, get_devices_by_user_id, update_device_last_seen,
};

// OplogEntry operations
pub use db::operations::{create_oplog_entry, get_oplog_entries_since};

// Peer operations
pub use db::operations::{create_peer, get_all_peers, get_peer, get_peers_by_user_id};

// ============================================================================
// Business Logic
// ============================================================================

// User management
pub use logic::{add_device_to_user, get_user_devices, login_user, register_user};

// Oplog entry builder helper
pub use logic::build_oplog_entry;

// ============================================================================
// P2P Synchronization
// ============================================================================

pub use logic::sync::{
    connect_to_bootstrap_nodes, connect_to_relay_servers, create_swarm, create_swarm_default,
    decode_sync_message, encode_sync_message, generate_device_id, handle_sync_message,
    parse_multiaddr_peer_id, update_peer_info, AhenkBehaviour, P2PConfig, SyncMessage,
};

// Sync manager for orchestrating P2P operations
pub use logic::sync_manager::SyncManager;

// ============================================================================
// Device Authorization
// ============================================================================

pub use auth::{
    create_auth_response, AuthChallenge, AuthResponse, AuthResult, AuthorizerWorkflow,
    DeviceAuthManager, NewDeviceWorkflow, PairingSession,
};

// ============================================================================
// CRDT Operations
// ============================================================================

pub use crdt::{local_apply, merge, HybridLogicalClock};

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AhenkError::Validation("test error".to_string());
        assert_eq!(format!("{}", err), "Validation error: test error");
    }

    #[test]
    fn test_error_from_string() {
        let err: AhenkError = "test".into();
        assert!(matches!(err, AhenkError::Other(_)));
    }
}
