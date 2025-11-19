-- ⚠️ DEPRECATED: This file is kept for reference only.
-- ⚠️ Active schema is managed through migrations in: src/db/migrations/
-- ⚠️ See: DATABASE_MIGRATIONS.md for migration system documentation
--
-- Database schema for Ahenk (Reference Only)

-- Users Table: Stores user account information.
CREATE TABLE users (
    user_id UUID PRIMARY KEY,
    user_name VARCHAR(255) UNIQUE NOT NULL,
    user_password VARCHAR(255) NOT NULL,
    user_mail VARCHAR(50) UNIQUE NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Devices Table: Stores information about user devices for synchronization.
CREATE TABLE devices (
    device_id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    device_type VARCHAR(10) NOT NULL,
    push_token TEXT,
    last_seen TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(user_id)
);

-- Task Lists Table: Stores user-created lists for tasks.
CREATE TABLE task_lists (
    list_id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    name VARCHAR(100) NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(user_id)
);

-- Tasks Table: Stores individual tasks.
CREATE TABLE tasks (
    task_id UUID PRIMARY KEY,
    list_id UUID NOT NULL,
    content TEXT NOT NULL,
    is_completed BOOLEAN NOT NULL DEFAULT FALSE,
    due_date DATE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP,
    FOREIGN KEY (list_id) REFERENCES task_lists(list_id)
);

-- Blocks Table: Represents blocks of time in the planner. (there will be a revise)
CREATE TABLE blocks (
    block_id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(user_id)
);

-- Task Blocks Table: Maps tasks to specific blocks of time (planner).
CREATE TABLE task_blocks (
    task_id UUID NOT NULL,
    block_id UUID NOT NULL,
    PRIMARY KEY (task_id, block_id),
    FOREIGN KEY (task_id) REFERENCES tasks(task_id),
    FOREIGN KEY (block_id) REFERENCES blocks(block_id)
);

-- Blocked Items Table: Stores apps/websites that are blocked during focus sessions.
CREATE TABLE blocked_items (
    item_id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    item_type VARCHAR(10) NOT NULL, -- 'app' or 'website'
    identifier VARCHAR(100) NOT NULL, -- e.g., 'com.instagram.android' or 'youtube.com'
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    FOREIGN KEY (user_id) REFERENCES users(user_id)
);

-- Sounds Table: Stores available soundscapes.
CREATE TABLE sounds (
    sound_id UUID PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    category VARCHAR(50),
    file_url VARCHAR(255) NOT NULL
);

-- Favorite Sounds Table: Maps users to their favorite sounds.
CREATE TABLE favorite_sounds (
    user_id UUID NOT NULL,
    sound_id UUID NOT NULL,
    PRIMARY KEY (user_id, sound_id),
    FOREIGN KEY (user_id) REFERENCES users(user_id),
    FOREIGN KEY (sound_id) REFERENCES sounds(sound_id)
);

-- Habits Table: Stores user-defined habits.
CREATE TABLE habits (
    habit_id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    name VARCHAR(25) NOT NULL,
    description VARCHAR(255),
    habit_cover VARCHAR(255),
    frequency_type VARCHAR(10) NOT NULL, -- 'daily', 'weekly'
    FOREIGN KEY (user_id) REFERENCES users(user_id)
);

-- Habit Entries Table: Logs completions of habits.
CREATE TABLE habit_entries (
    entry_id UUID PRIMARY KEY,
    habit_id UUID NOT NULL,
    completion_date DATE NOT NULL,
    notes TEXT,
    FOREIGN KEY (habit_id) REFERENCES habits(habit_id)
);

-- Pomodoros Table: Stores user-configured Pomodoro timers.
CREATE TABLE pomodoros (
    pomodoro_id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    pomodoro_name VARCHAR(25) NOT NULL,
    pomodoro_cover VARCHAR(255),
    work_duration INTEGER NOT NULL,
    short_break_duration INTEGER NOT NULL,
    long_break_duration INTEGER NOT NULL,
    long_break_interval INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(user_id)
);

-- Oplog Table: Stores every operation as a CRDT entry for synchronization.
CREATE TABLE oplog (
    id UUID PRIMARY KEY,
    device_id UUID NOT NULL,
    timestamp BIGINT NOT NULL,
    table_name TEXT NOT NULL,
    op_type TEXT NOT NULL,
    data TEXT NOT NULL, -- JSON blob
    FOREIGN KEY (device_id) REFERENCES devices(device_id)
);

-- Peers Table: Stores information about other trusted devices in the sync network.
CREATE TABLE peers (
    peer_id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    device_id UUID NOT NULL,
    last_known_ip VARCHAR(255),
    last_sync_time TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(user_id),
    FOREIGN KEY (device_id) REFERENCES devices(device_id)
);
