//! Tauri API module - only available with the "tauri-api" feature
//!
//! This module provides Tauri command handlers for the Ahenk sync engine.
//!
//! **Note**: This module only exposes core sync infrastructure (User, Device, P2P).
//! Application-specific features (Tasks, Habits, Pomodoros, etc.) should be
//! implemented in the consuming application using Ahenk as a sync backend.

#[cfg(feature = "tauri-api")]
mod tauri_commands {
    use crate::logic::sync_manager::SyncManager;
    use crate::logic::{login_user, register_user};
    use crate::models::User;
    use chrono::{DateTime, Utc};
    use rusqlite::Connection;
    use std::sync::Arc;
    use std::sync::Mutex;
    use tauri::State;
    use uuid::Uuid;

    /// Database connection wrapper for Tauri state management
    ///
    /// # TODO: Add persistent device_id
    /// Currently each command generates a new device_id which breaks CRDT sync.
    /// Should be:
    /// ```rust
    /// pub struct AppState {
    ///     pub conn: Mutex<Connection>,
    ///     pub device_id: Uuid,  // Persistent device ID for this app instance
    /// }
    /// ```
    pub struct DbConnection(pub Mutex<Connection>);

    // ============================================================================
    // User Management
    // ============================================================================

    /// Register a new user account
    ///
    /// # Arguments
    /// * `username` - Unique username
    /// * `email` - Unique email address
    /// * `password` - User password (will be hashed with Argon2)
    #[tauri::command]
    pub fn ahenk_register_user(
        username: String,
        email: String,
        password: String,
        conn: State<DbConnection>,
    ) -> Result<User, String> {
        let mut db = conn.0.lock().map_err(|e| e.to_string())?;
        register_user(&mut db, username, email, password).map_err(|e| e.to_string())
    }

    /// Authenticate a user
    ///
    /// # Arguments
    /// * `username` - Username or email
    /// * `password` - User password
    #[tauri::command]
    pub fn ahenk_login_user(
        username: String,
        password: String,
        conn: State<DbConnection>,
    ) -> Result<User, String> {
        let db = conn.0.lock().map_err(|e| e.to_string())?;
        login_user(&db, &username, &password).map_err(|e| e.to_string())
    }

    // ============================================================================
    // P2P Synchronization
    // ============================================================================

    /// Get current synchronization status
    ///
    /// Returns: (is_syncing, last_sync_time, connected_peers, pending_changes_count, is_online)
    #[tauri::command]
    pub fn ahenk_get_sync_status(
        sync_manager_state: tauri::State<Arc<Mutex<SyncManager>>>,
    ) -> Result<(bool, Option<DateTime<Utc>>, Vec<String>, usize, bool), String> {
        let sync_manager = sync_manager_state
            .inner()
            .lock()
            .map_err(|e| e.to_string())?;
        Ok((
            sync_manager.get_is_syncing(),
            sync_manager.get_last_sync_time(),
            sync_manager.get_connected_peers(),
            sync_manager.get_pending_changes_count(),
            sync_manager.is_online,
        ))
    }

    /// Request immediate synchronization with peers
    ///
    /// # Arguments
    /// * `user_id` - User UUID
    #[tauri::command]
    pub fn ahenk_request_sync(
        user_id: String,
        sync_manager_state: tauri::State<Arc<Mutex<SyncManager>>>,
    ) -> Result<(), String> {
        let mut sync_manager = sync_manager_state
            .inner()
            .lock()
            .map_err(|e| e.to_string())?;
        let _user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;

        // Request sync from last known time or epoch
        let since = sync_manager.get_last_sync_time().unwrap_or_else(Utc::now);
        sync_manager.request_sync(since).map_err(|e| e.to_string())
    }

    /// Set device online/offline status
    ///
    /// # Arguments
    /// * `is_online` - Whether the device should actively sync
    #[tauri::command]
    pub fn ahenk_set_online_status(
        is_online: bool,
        sync_manager_state: tauri::State<Arc<Mutex<SyncManager>>>,
    ) -> Result<(), String> {
        let mut sync_manager = sync_manager_state
            .inner()
            .lock()
            .map_err(|e| e.to_string())?;
        sync_manager
            .set_online_status(is_online)
            .map_err(|e| e.to_string())
    }

    /// Sync all pending changes accumulated while offline
    #[tauri::command]
    pub fn ahenk_sync_pending_changes(
        sync_manager_state: tauri::State<Arc<Mutex<SyncManager>>>,
    ) -> Result<(), String> {
        let mut sync_manager = sync_manager_state
            .inner()
            .lock()
            .map_err(|e| e.to_string())?;
        sync_manager
            .sync_pending_changes()
            .map_err(|e| e.to_string())
    }
}

#[cfg(feature = "tauri-api")]
pub use tauri_commands::*;
