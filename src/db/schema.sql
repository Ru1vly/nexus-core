-- ⚠️ DEPRECATED: This file is kept for reference only.
-- ⚠️ Active schema is managed through migrations in: src/db/migrations/
-- ⚠️ See: DATABASE_MIGRATIONS.md for migration system documentation
--
-- Ahenk Core Database Schema (Reference Only)
-- Version: 0.1.0
-- This represents the schema created by migration 001_initial_schema.sql

-- Users Table: Account information and authentication
CREATE TABLE users (
    user_id TEXT PRIMARY KEY,            -- UUID as TEXT for SQLite compatibility
    user_name TEXT UNIQUE NOT NULL,      -- Unique username
    user_password TEXT NOT NULL,         -- Argon2 hashed password
    user_mail TEXT UNIQUE NOT NULL,      -- Unique email address
    created_at TEXT NOT NULL             -- RFC3339 timestamp
);

-- Devices Table: Multi-device registry for synchronization
CREATE TABLE devices (
    device_id TEXT PRIMARY KEY,          -- UUID as TEXT
    user_id TEXT NOT NULL,               -- Owner of this device
    device_type TEXT NOT NULL,           -- e.g., "phone", "laptop", "tablet"
    push_token TEXT,                     -- Optional: For push notifications
    last_seen TEXT,                      -- RFC3339 timestamp of last activity
    FOREIGN KEY (user_id) REFERENCES users(user_id)
);

-- Operation Log (OpLog): CRDT change tracking for synchronization
CREATE TABLE oplog (
    id TEXT PRIMARY KEY,                 -- UUID as TEXT
    device_id TEXT NOT NULL,             -- Device that created this operation
    timestamp INTEGER NOT NULL,          -- HLC (Hybrid Logical Clock) timestamp
    table_name TEXT NOT NULL,            -- Which table this operation affects
    op_type TEXT NOT NULL,               -- Operation type: "create", "update", "delete"
    data TEXT NOT NULL,                  -- JSON-serialized operation data
    FOREIGN KEY (device_id) REFERENCES devices(device_id)
);

-- Index for efficient oplog queries (sync since timestamp)
CREATE INDEX idx_oplog_timestamp ON oplog(timestamp);

-- Peers Table: P2P network peer information
CREATE TABLE peers (
    peer_id TEXT PRIMARY KEY,            -- libp2p PeerId
    user_id TEXT NOT NULL,               -- User this peer belongs to
    device_id TEXT NOT NULL,             -- Device identifier
    last_known_ip TEXT,                  -- Last known IP address (optional)
    last_sync_time TEXT,                 -- RFC3339 timestamp of last sync
    FOREIGN KEY (user_id) REFERENCES users(user_id),
    FOREIGN KEY (device_id) REFERENCES devices(device_id)
);

-- ============================================================================
-- NOTES FOR APPLICATION DEVELOPERS
-- ============================================================================
--
-- Ahenk provides only the core sync infrastructure tables above.
-- Your application should create additional tables for your domain models.
--
-- Example: Adding a Tasks table to your application
--
-- CREATE TABLE tasks (
--     task_id TEXT PRIMARY KEY,
--     user_id TEXT NOT NULL,
--     content TEXT NOT NULL,
--     completed INTEGER NOT NULL DEFAULT 0,  -- SQLite uses 0/1 for boolean
--     due_date TEXT,                         -- Optional RFC3339 date
--     created_at TEXT NOT NULL,
--     FOREIGN KEY (user_id) REFERENCES users(user_id)
-- );
--
-- CREATE INDEX idx_tasks_user ON tasks(user_id);
-- CREATE INDEX idx_tasks_due_date ON tasks(due_date);
--
-- Then use Ahenk's CRDT functions to sync your tasks:
--
-- ```rust
-- use ahenk::{build_oplog_entry, local_apply};
--
-- // Create task in your table
-- conn.execute("INSERT INTO tasks (...) VALUES (...)", params![...])?;
--
-- // Create oplog entry for sync
-- let oplog = build_oplog_entry(device_id, "tasks", "create", &task)?;
-- local_apply(&mut conn, &oplog)?;
-- ```
--
-- ============================================================================
