//! Business logic for ahenk synchronization infrastructure.
//!
//! This module provides high-level functions for:
//! - User registration and authentication
//! - Device management and authorization
//! - P2P synchronization (see sync module)
//! - Sync orchestration (see sync_manager module)
//!
//! # TODO: Error Handling Migration
//! Currently this module uses `Result<T, String>` for error handling.
//! Should be migrated to `Result<T, AhenkError>` for better error categorization
//! and consistent error handling across the crate.

pub mod sync;
pub mod sync_manager;

use crate::crdt;
use crate::db::operations;
use crate::models::{Device, OplogEntry, User};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::Utc;
use rusqlite::Connection;
use serde::Serialize;
use uuid::Uuid;

/// Helper function to build an oplog entry for CRDT synchronization
pub fn build_oplog_entry<T: Serialize>(
    device_id: Uuid,
    table: &str,
    op_type: &str,
    value: &T,
) -> Result<OplogEntry, String> {
    let data = serde_json::to_value(value)
        .map_err(|e| format!("Failed to serialize {} payload: {}", table, e))?;

    Ok(OplogEntry {
        id: Uuid::new_v4(),
        device_id,
        timestamp: crdt::HybridLogicalClock::now().to_timestamp(),
        table: table.to_string(),
        op_type: op_type.to_string(),
        data,
    })
}

// ============================================================================
// User Management
// ============================================================================

/// Registers a new user after validating uniqueness and hashing their password.
///
/// # Arguments
/// * `conn` - Database connection
/// * `user_name` - Username (must be unique)
/// * `user_mail` - Email address (must be unique)
/// * `password` - Plain text password (will be hashed with Argon2)
///
/// # Returns
/// * `Ok(User)` - The created user
/// * `Err(String)` - Validation or database error
pub fn register_user(
    conn: &Connection,
    user_name: String,
    user_mail: String,
    password: String,
) -> Result<User, String> {
    // Validate inputs
    let normalized_name = user_name.trim();
    if normalized_name.is_empty() {
        return Err("Username cannot be empty".to_string());
    }

    let normalized_mail = user_mail.trim().to_lowercase();
    if normalized_mail.is_empty() {
        return Err("Email cannot be empty".to_string());
    }

    if password.trim().is_empty() {
        return Err("Password cannot be empty".to_string());
    }

    // Check for existing user
    if operations::get_user_by_name(conn, normalized_name)
        .map_err(|e| format!("Database error: {}", e))?
        .is_some()
    {
        return Err("Username already exists".to_string());
    }

    if operations::get_user_by_mail(conn, &normalized_mail)
        .map_err(|e| format!("Database error: {}", e))?
        .is_some()
    {
        return Err("Email already registered".to_string());
    }

    // Hash password with Argon2
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("Password hashing failed: {}", e))?
        .to_string();

    // Create user
    let new_user = User {
        user_id: Uuid::new_v4(),
        user_name: normalized_name.to_string(),
        user_password_hash: password_hash,
        user_mail: normalized_mail,
        created_at: Utc::now(),
    };

    operations::create_user(conn, &new_user)
        .map_err(|e| format!("Failed to create user: {}", e))?;

    Ok(new_user)
}

/// Validates credentials using Argon2 and returns the matching user record.
///
/// # Arguments
/// * `conn` - Database connection
/// * `identifier` - Username or email
/// * `password` - Plain text password
///
/// # Returns
/// * `Ok(User)` - The authenticated user
/// * `Err(String)` - Authentication failed or database error
pub fn login_user(conn: &Connection, identifier: &str, password: &str) -> Result<User, String> {
    let trimmed_identifier = identifier.trim();
    if trimmed_identifier.is_empty() {
        return Err("Identifier cannot be empty".to_string());
    }

    if password.is_empty() {
        return Err("Password cannot be empty".to_string());
    }

    // Try username first, then email
    let user_result = operations::get_user_by_name(conn, trimmed_identifier)
        .map_err(|e| format!("Database error: {}", e))?;

    let user = match user_result {
        Some(user) => user,
        None => {
            let email_lookup =
                operations::get_user_by_mail(conn, &trimmed_identifier.to_lowercase())
                    .map_err(|e| format!("Database error: {}", e))?;
            email_lookup.ok_or_else(|| "Invalid credentials".to_string())?
        }
    };

    // Verify password using timing-safe comparison
    let parsed_hash = PasswordHash::new(&user.user_password_hash)
        .map_err(|_| "Stored password hash is invalid".to_string())?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| "Invalid credentials".to_string())?;

    Ok(user)
}

// ============================================================================
// Device Management
// ============================================================================

/// Associates a new device with an existing user.
///
/// # Arguments
/// * `conn` - Database connection
/// * `user_id` - User UUID
/// * `device_type` - Device type (e.g., "ios", "android", "desktop")
/// * `push_token` - Optional push notification token
///
/// # Returns
/// * `Ok(Device)` - The created device
/// * `Err(String)` - Validation or database error
pub fn add_device_to_user(
    conn: &Connection,
    user_id: Uuid,
    device_type: String,
    push_token: Option<String>,
) -> Result<Device, String> {
    // Verify user exists
    operations::get_user(conn, user_id)
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "User not found".to_string())?;

    // Validate device type
    let trimmed_type = device_type.trim();
    if trimmed_type.is_empty() {
        return Err("Device type cannot be empty".to_string());
    }

    // Normalize push token
    let normalized_push_token = push_token.and_then(|token| {
        let trimmed = token.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });

    // Create device
    let new_device = Device {
        device_id: Uuid::new_v4(),
        user_id,
        device_type: trimmed_type.to_string(),
        push_token: normalized_push_token,
        last_seen: Some(Utc::now()),
    };

    operations::create_device(conn, &new_device)
        .map_err(|e| format!("Failed to create device: {}", e))?;

    Ok(new_device)
}

/// Retrieves all devices currently associated with the user.
///
/// # Arguments
/// * `conn` - Database connection
/// * `user_id` - User UUID
///
/// # Returns
/// * `Ok(Vec<Device>)` - List of devices
/// * `Err(String)` - Database error
pub fn get_user_devices(conn: &Connection, user_id: Uuid) -> Result<Vec<Device>, String> {
    operations::get_devices_by_user_id(conn, user_id).map_err(|e| format!("Database error: {}", e))
}
