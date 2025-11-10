//! # Nexus Core
//!
//! Core library for the FocusSuite nexus system, providing database operations,
//! business logic, and peer-to-peer synchronization capabilities.
//!
//! ## Modules
//!
//! - `models`: Data structures for users, tasks, habits, and other entities
//! - `db`: Database operations and initialization
//! - `logic`: Business logic for user management, tasks, habits, etc.
//! - `sync`: Peer-to-peer synchronization using libp2p
//! - `error`: Error types and result aliases
//!
//! ## Example
//!
//! ```rust,no_run
//! use nexus_core::{initialize_database, logic, models::User};
//!
//! // Initialize database
//! let conn = initialize_database("nexus.db").unwrap();
//!
//! // Register a new user
//! let user = logic::register_user(
//!     &conn,
//!     "alice".to_string(),
//!     "alice@example.com".to_string(),
//!     "secure_password".to_string(),
//! ).unwrap();
//! ```

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

// Re-export error types for convenient usage
pub use error::{NexusError, Result};

// Re-export all model types
pub use models::{
    Block, BlockedItem, Device, FavoriteSound, Habit, HabitEntry, OplogEntry, Peer, Pomodoro,
    Sound, Task, TaskBlock, TaskList, User,
};

// Re-export database migration functions
pub use db::migrations::{apply_migrations, get_current_version, get_migration_history};

// Re-export key database operations
pub use db::operations::{
    // Block operations
    create_block,
    // Blocked items operations
    create_blocked_item,
    // Device operations
    create_device,
    create_favorite_sound,
    // Habit operations
    create_habit,
    create_habit_entry,
    // Oplog operations
    create_oplog_entry,
    // Peer operations
    create_peer,
    // Pomodoro operations
    create_pomodoro,
    // Sound operations
    create_sound,
    // Task operations
    create_task,
    create_task_block,
    // Task list operations
    create_task_list,
    // User operations
    create_user,
    delete_favorite_sound,
    get_active_blocked_items_by_user_id,
    get_all_peers,
    get_all_sounds,
    get_block,
    get_device,
    get_devices_by_user_id,
    get_favorite_sounds_by_user_id,
    get_habit,
    get_habit_entries_sorted_by_date,
    get_oplog_entries_since,
    get_peer,
    get_peers_by_user_id,
    get_pomodoros_by_user_id,
    get_sound,
    get_sounds_by_category,
    get_task,
    get_task_block,
    get_task_list,
    get_task_lists_by_user_id,
    get_tasks_by_block_id,
    get_tasks_by_list_id,
    get_tasks_due_on_date_for_user,
    get_user,
    get_user_by_mail,
    get_user_by_name,
    initialize_database,
    update_device_last_seen,
    update_task_status,
};

// Re-export key business logic functions
pub use logic::{
    add_device_to_user,
    // Blocklist management
    add_item_to_blocklist,
    // Task management
    add_task_to_list,
    // Note: apply_oplog_entry removed - use crdt::local_apply instead
    assign_task_to_block,
    // Habit management
    create_habit as create_new_habit,
    create_new_task_list,
    get_active_blocklist,
    get_all_pomodoro_presets,
    // Task list management
    get_all_task_lists_for_user,
    get_all_tasks_in_list,
    get_habit_streak,
    get_tasks_due_today,
    get_tasks_for_a_specific_block,
    get_user_devices,
    log_habit_completion,
    login_user,
    mark_task_as_complete,
    // User management
    register_user,
    // Pomodoro presets
    save_pomodoro_preset,
    // Time blocking
    schedule_block,
};

// Re-export P2P sync types and functions
pub use logic::sync::{
    NexusBehaviour, P2PConfig, SyncMessage, connect_to_bootstrap_nodes, connect_to_relay_servers,
    create_swarm, create_swarm_default, decode_sync_message, encode_sync_message,
    generate_device_id, handle_sync_message, parse_multiaddr_peer_id, update_peer_info,
};

// Re-export sync manager
pub use logic::sync_manager::SyncManager;

// Re-export device authorization types and workflows
pub use auth::{
    AuthChallenge, AuthResponse, AuthResult, AuthorizerWorkflow, DeviceAuthManager,
    NewDeviceWorkflow, PairingSession, create_auth_response,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = NexusError::Validation("test error".to_string());
        assert_eq!(format!("{}", err), "Validation error: test error");
    }

    #[test]
    fn test_error_from_string() {
        let err: NexusError = "test".into();
        assert!(matches!(err, NexusError::Other(_)));
    }
}
