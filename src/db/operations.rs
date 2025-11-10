use crate::models::{
    Block, BlockedItem, Device, FavoriteSound, Habit, HabitEntry, OplogEntry, Peer, Pomodoro,
    PomodoroSession, Sound, Soundscape, Task, TaskBlock, TaskList, User, UserPreference,
};
use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{Connection, Result, Row, params, types::Type};
use uuid::Uuid;

pub fn initialize_database(db_path: &str) -> Result<Connection> {
    let conn = Connection::open(db_path)?;

    // Apply all pending migrations as operations
    // This will create tables if they don't exist (new database)
    // or upgrade the schema to the latest version (existing database)
    super::migrations::apply_migrations(&conn)?;

    Ok(conn)
}

fn conversion_failure<E>(column_index: usize, err: E) -> rusqlite::Error
where
    E: std::error::Error + Send + Sync + 'static,
{
    rusqlite::Error::FromSqlConversionFailure(column_index, Type::Text, Box::new(err))
}

fn parse_uuid_column(row: &Row, idx: usize) -> rusqlite::Result<Uuid> {
    let value: String = row.get(idx)?;
    Uuid::parse_str(&value).map_err(|e| conversion_failure(idx, e))
}

fn parse_datetime_column(row: &Row, idx: usize) -> rusqlite::Result<DateTime<Utc>> {
    let value: String = row.get(idx)?;
    DateTime::parse_from_rfc3339(&value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| conversion_failure(idx, e))
}

fn parse_optional_datetime_column(
    row: &Row,
    idx: usize,
) -> rusqlite::Result<Option<DateTime<Utc>>> {
    let value: Option<String> = row.get(idx)?;
    match value {
        Some(raw) => DateTime::parse_from_rfc3339(&raw)
            .map(|dt| dt.with_timezone(&Utc))
            .map(Some)
            .map_err(|e| conversion_failure(idx, e)),
        None => Ok(None),
    }
}

fn parse_naive_date_column(row: &Row, idx: usize) -> rusqlite::Result<NaiveDate> {
    let value: String = row.get(idx)?;
    NaiveDate::parse_from_str(&value, "%Y-%m-%d").map_err(|e| conversion_failure(idx, e))
}

fn parse_optional_naive_date_column(row: &Row, idx: usize) -> rusqlite::Result<Option<NaiveDate>> {
    let value: Option<String> = row.get(idx)?;
    match value {
        Some(raw) => NaiveDate::parse_from_str(&raw, "%Y-%m-%d")
            .map(Some)
            .map_err(|e| conversion_failure(idx, e)),
        None => Ok(None),
    }
}

// Helper functions for row mapping
fn row_to_user(row: &Row) -> rusqlite::Result<User> {
    Ok(User {
        user_id: parse_uuid_column(row, 0)?,
        user_name: row.get(1)?,
        user_password_hash: row.get(2)?,
        user_mail: row.get(3)?,
        created_at: parse_datetime_column(row, 4)?,
    })
}

fn row_to_device(row: &Row) -> rusqlite::Result<Device> {
    Ok(Device {
        device_id: parse_uuid_column(row, 0)?,
        user_id: parse_uuid_column(row, 1)?,
        device_type: row.get(2)?,
        push_token: row.get(3)?,
        last_seen: parse_optional_datetime_column(row, 4)?,
    })
}
fn row_to_task_list(row: &Row) -> rusqlite::Result<TaskList> {
    Ok(TaskList {
        list_id: parse_uuid_column(row, 0)?,
        user_id: parse_uuid_column(row, 1)?,
        name: row.get(2)?,
    })
}

fn row_to_task(row: &Row) -> rusqlite::Result<Task> {
    Ok(Task {
        task_id: parse_uuid_column(row, 0)?,
        list_id: parse_uuid_column(row, 1)?,
        content: row.get(2)?,
        is_completed: row.get(3)?,
        due_date: parse_optional_naive_date_column(row, 4)?,
        created_at: parse_datetime_column(row, 5)?,
        updated_at: parse_optional_datetime_column(row, 6)?,
    })
}
fn row_to_habit(row: &Row) -> rusqlite::Result<Habit> {
    Ok(Habit {
        habit_id: parse_uuid_column(row, 0)?,
        user_id: parse_uuid_column(row, 1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        habit_cover: row.get(4)?,
        frequency_type: row.get(5)?,
    })
}

fn row_to_habit_entry(row: &Row) -> rusqlite::Result<HabitEntry> {
    Ok(HabitEntry {
        entry_id: parse_uuid_column(row, 0)?,
        habit_id: parse_uuid_column(row, 1)?,
        completion_date: parse_naive_date_column(row, 2)?,
        notes: row.get(3)?,
    })
}

fn row_to_block(row: &Row) -> rusqlite::Result<Block> {
    Ok(Block {
        block_id: parse_uuid_column(row, 0)?,
        user_id: parse_uuid_column(row, 1)?,
        start_time: parse_datetime_column(row, 2)?,
        end_time: parse_datetime_column(row, 3)?,
    })
}

fn row_to_task_block(row: &Row) -> rusqlite::Result<TaskBlock> {
    Ok(TaskBlock {
        task_id: parse_uuid_column(row, 0)?,
        block_id: parse_uuid_column(row, 1)?,
    })
}

fn row_to_pomodoro(row: &Row) -> rusqlite::Result<Pomodoro> {
    Ok(Pomodoro {
        pomodoro_id: parse_uuid_column(row, 0)?,
        user_id: parse_uuid_column(row, 1)?,
        pomodoro_name: row.get(2)?,
        pomodoro_cover: row.get(3)?,
        work_duration: row.get(4)?,
        short_break_duration: row.get(5)?,
        long_break_duration: row.get(6)?,
        long_break_interval: row.get(7)?,
        created_at: parse_datetime_column(row, 8)?,
        updated_at: parse_optional_datetime_column(row, 9)?,
    })
}

fn row_to_pomodoro_session(row: &Row) -> rusqlite::Result<PomodoroSession> {
    Ok(PomodoroSession {
        session_id: parse_uuid_column(row, 0)?,
        user_id: parse_uuid_column(row, 1)?,
        pomodoro_id: row
            .get::<_, Option<String>>(2)?
            .map(|s| Uuid::parse_str(&s))
            .transpose()
            .map_err(|e| conversion_failure(2, e))?,
        session_type: row.get(3)?,
        duration_seconds: row.get(4)?,
        completed: row.get(5)?,
        started_at: parse_datetime_column(row, 6)?,
        completed_at: parse_datetime_column(row, 7)?,
        notes: row.get(8)?,
    })
}

fn row_to_blocked_item(row: &Row) -> rusqlite::Result<BlockedItem> {
    Ok(BlockedItem {
        item_id: parse_uuid_column(row, 0)?,
        user_id: parse_uuid_column(row, 1)?,
        item_type: row.get(2)?,
        identifier: row.get(3)?,
        is_active: row.get(4)?,
    })
}

fn row_to_sound(row: &Row) -> rusqlite::Result<Sound> {
    Ok(Sound {
        sound_id: parse_uuid_column(row, 0)?,
        name: row.get(1)?,
        category: row.get(2)?,
        file_url: row.get(3)?,
    })
}

fn row_to_soundscape(row: &Row) -> rusqlite::Result<Soundscape> {
    Ok(Soundscape {
        soundscape_id: parse_uuid_column(row, 0)?,
        user_id: parse_uuid_column(row, 1)?,
        name: row.get(2)?,
        file_path: row.get(3)?,
        volume: row.get(4)?,
        is_playing: row.get(5)?,
    })
}

// User CRUD
pub fn create_user(conn: &Connection, user: &User) -> Result<()> {
    conn.execute("INSERT INTO users (user_id, user_name, user_password, user_mail, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![&user.user_id.to_string(), &user.user_name, &user.user_password_hash, &user.user_mail, &user.created_at.to_rfc3339()])?;
    Ok(())
}

// User READ (by user_id)
pub fn get_user(conn: &Connection, user_id: Uuid) -> Result<Option<User>> {
    let mut stmt = conn.prepare("SELECT user_id, user_name, user_password, user_mail, created_at FROM users WHERE user_id = ?1")?;
    let mut rows = stmt.query_map(params![user_id.to_string()], row_to_user)?;
    rows.next().transpose()
}

// User READ (by user_name)
pub fn get_user_by_name(conn: &Connection, user_name: &str) -> Result<Option<User>> {
    let mut stmt = conn.prepare("SELECT user_id, user_name, user_password, user_mail, created_at FROM users WHERE user_name = ?1")?;
    let mut rows = stmt.query_map(params![user_name], row_to_user)?;
    rows.next().transpose()
}

// User READ (by user_mail)
pub fn get_user_by_mail(conn: &Connection, user_mail: &str) -> Result<Option<User>> {
    let mut stmt = conn.prepare("SELECT user_id, user_name, user_password, user_mail, created_at FROM users WHERE user_mail = ?1")?;
    let mut rows = stmt.query_map(params![user_mail], row_to_user)?;
    rows.next().transpose()
}

// Device CREATE
pub fn create_device(conn: &Connection, device: &Device) -> Result<()> {
    conn.execute(
        "INSERT INTO devices (device_id, user_id, device_type, push_token, last_seen) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            &device.device_id.to_string(),
            &device.user_id.to_string(),
            &device.device_type,
            &device.push_token,
            &device.last_seen.map(|dt| dt.to_rfc3339())
        ],
    )?;
    Ok(())
}

// Device READ (by device_id)
pub fn get_device(conn: &Connection, device_id: Uuid) -> Result<Option<Device>> {
    let mut stmt = conn.prepare("SELECT device_id, user_id, device_type, push_token, last_seen FROM devices WHERE device_id = ?1")?;
    let mut rows = stmt.query_map(params![device_id.to_string()], row_to_device)?;
    rows.next().transpose()
}

// Device READ (by user_id)
pub fn get_devices_by_user_id(conn: &Connection, user_id: Uuid) -> Result<Vec<Device>> {
    let mut stmt = conn.prepare("SELECT device_id, user_id, device_type, push_token, last_seen FROM devices WHERE user_id = ?1")?;
    let rows = stmt.query_map(params![user_id.to_string()], row_to_device)?;

    let mut devices = Vec::new();
    for row in rows {
        devices.push(row?);
    }

    Ok(devices)
}

// Device UPDATE (last_seen)
pub fn update_device_last_seen(
    conn: &Connection,
    device_id: Uuid,
    last_seen: DateTime<Utc>,
) -> Result<usize> {
    conn.execute(
        "UPDATE devices SET last_seen = ?1 WHERE device_id = ?2",
        params![last_seen.to_rfc3339(), device_id.to_string()],
    )
}
// TaskList CREATE
pub fn create_task_list(conn: &Connection, task_list: &TaskList) -> Result<()> {
    conn.execute(
        "INSERT INTO task_lists (list_id, user_id, name) VALUES (?1, ?2, ?3)",
        params![
            &task_list.list_id.to_string(),
            &task_list.user_id.to_string(),
            &task_list.name
        ],
    )?;
    Ok(())
}

// Task CREATE
pub fn create_task(conn: &Connection, task: &Task) -> Result<()> {
    conn.execute(
        "INSERT INTO tasks (task_id, list_id, content, is_completed, due_date, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            &task.task_id.to_string(),
            &task.list_id.to_string(),
            &task.content,
            &task.is_completed,
            &task.due_date.map(|d| d.to_string()),
            &task.created_at.to_rfc3339(),
            &task.updated_at.map(|dt| dt.to_rfc3339()),
        ],
    )?;
    Ok(())
}

// TaskList READ (by list_id)
pub fn get_task_list(conn: &Connection, list_id: Uuid) -> Result<Option<TaskList>> {
    let mut stmt =
        conn.prepare("SELECT list_id, user_id, name FROM task_lists WHERE list_id = ?1")?;
    let mut rows = stmt.query_map(params![list_id.to_string()], row_to_task_list)?;
    rows.next().transpose()
}
// Task UPDATE
pub fn update_task_status(conn: &Connection, task_id: Uuid, is_completed: bool) -> Result<usize> {
    conn.execute(
        "UPDATE tasks SET is_completed = ?1 WHERE task_id = ?2",
        params![is_completed, task_id.to_string()],
    )
}

// Task READ (by task_id)
pub fn get_task(conn: &Connection, task_id: Uuid) -> Result<Option<Task>> {
    let mut stmt = conn.prepare("SELECT task_id, list_id, content, is_completed, due_date, created_at, updated_at FROM tasks WHERE task_id = ?1")?;
    let mut rows = stmt.query_map(params![task_id.to_string()], row_to_task)?;
    rows.next().transpose()
}
// Task READ (by list_id)
pub fn get_tasks_by_list_id(conn: &Connection, list_id: Uuid) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare("SELECT task_id, list_id, content, is_completed, due_date, created_at, updated_at FROM tasks WHERE list_id = ?1")?;
    let rows = stmt.query_map(params![list_id.to_string()], row_to_task)?;

    let mut tasks = Vec::new();
    for row in rows {
        tasks.push(row?);
    }

    Ok(tasks)
}
// Task READ (by user and due date)
pub fn get_tasks_due_on_date_for_user(
    conn: &Connection,
    user_id: Uuid,
    date: NaiveDate,
) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare(
        "
        SELECT t.task_id, t.list_id, t.content, t.is_completed, t.due_date, t.created_at, t.updated_at
        FROM tasks t
        INNER JOIN task_lists tl ON t.list_id = tl.list_id
        WHERE tl.user_id = ?1 AND t.due_date = ?2
    ",
    )?;

    let rows = stmt.query_map(params![user_id.to_string(), date.to_string()], row_to_task)?;

    let mut tasks = Vec::new();
    for row in rows {
        tasks.push(row?);
    }

    Ok(tasks)
}
// Habit CREATE
pub fn create_habit(conn: &Connection, habit: &Habit) -> Result<()> {
    conn.execute(
        "INSERT INTO habits (habit_id, user_id, name, description, habit_cover, frequency_type) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![&habit.habit_id.to_string(), &habit.user_id.to_string(), &habit.name, &habit.description, &habit.habit_cover, &habit.frequency_type],
    )?;
    Ok(())
}

// HabitEntry CREATE
pub fn create_habit_entry(conn: &Connection, habit_entry: &HabitEntry) -> Result<()> {
    conn.execute(
        "INSERT INTO habit_entries (entry_id, habit_id, completion_date, notes) VALUES (?1, ?2, ?3, ?4)",
        params![&habit_entry.entry_id.to_string(), &habit_entry.habit_id.to_string(), &habit_entry.completion_date.to_string(), &habit_entry.notes],
    )?;
    Ok(())
}

// Habit READ (by habit_id)
pub fn get_habit(conn: &Connection, habit_id: Uuid) -> Result<Option<Habit>> {
    let mut stmt = conn.prepare("SELECT habit_id, user_id, name, description, habit_cover, frequency_type FROM habits WHERE habit_id = ?1")?;
    let mut rows = stmt.query_map(params![habit_id.to_string()], row_to_habit)?;
    rows.next().transpose()
}

pub fn get_habits_by_user_id(conn: &Connection, user_id: Uuid) -> Result<Vec<Habit>> {
    let mut stmt = conn.prepare(
        "SELECT habit_id, user_id, name, description, habit_cover, frequency_type FROM habits WHERE user_id = ?1 ORDER BY name",
    )?;
    let rows = stmt.query_map(params![user_id.to_string()], row_to_habit)?;

    let mut habits = Vec::new();
    for row in rows {
        habits.push(row?);
    }

    Ok(habits)
}
// HabitEntry READ (by habit_id, sorted)
pub fn get_habit_entries_sorted_by_date(
    conn: &Connection,
    habit_id: Uuid,
) -> Result<Vec<HabitEntry>> {
    let mut stmt = conn.prepare("SELECT entry_id, habit_id, completion_date, notes FROM habit_entries WHERE habit_id = ?1 ORDER BY completion_date DESC")?;
    let rows = stmt.query_map(params![habit_id.to_string()], row_to_habit_entry)?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }

    Ok(entries)
}
// Block CREATE
pub fn create_block(conn: &Connection, block: &Block) -> Result<()> {
    conn.execute(
        "INSERT INTO blocks (block_id, user_id, start_time, end_time) VALUES (?1, ?2, ?3, ?4)",
        params![
            &block.block_id.to_string(),
            &block.user_id.to_string(),
            &block.start_time.to_rfc3339(),
            &block.end_time.to_rfc3339()
        ],
    )?;
    Ok(())
}

// TaskBlock CREATE
pub fn create_task_block(conn: &Connection, task_block: &TaskBlock) -> Result<()> {
    conn.execute(
        "INSERT INTO task_blocks (task_id, block_id) VALUES (?1, ?2)",
        params![
            &task_block.task_id.to_string(),
            &task_block.block_id.to_string()
        ],
    )?;
    Ok(())
}

// Block READ (by block_id)
pub fn get_block(conn: &Connection, block_id: Uuid) -> Result<Option<Block>> {
    let mut stmt = conn.prepare(
        "SELECT block_id, user_id, start_time, end_time FROM blocks WHERE block_id = ?1",
    )?;
    let mut rows = stmt.query_map(params![block_id.to_string()], row_to_block)?;
    rows.next().transpose()
}

// TaskBlock READ (by task_id and block_id)
pub fn get_task_block(
    conn: &Connection,
    task_id: Uuid,
    block_id: Uuid,
) -> Result<Option<TaskBlock>> {
    let mut stmt = conn.prepare(
        "SELECT task_id, block_id FROM task_blocks WHERE task_id = ?1 AND block_id = ?2",
    )?;
    let mut rows = stmt.query_map(
        params![task_id.to_string(), block_id.to_string()],
        row_to_task_block,
    )?;
    rows.next().transpose()
}
// Task READ (by block_id)
pub fn get_tasks_by_block_id(conn: &Connection, block_id: Uuid) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare(
        "
        SELECT t.task_id, t.list_id, t.content, t.is_completed, t.due_date, t.created_at, t.updated_at
        FROM tasks t
        INNER JOIN task_blocks tb ON t.task_id = tb.task_id
        WHERE tb.block_id = ?1
    ",
    )?;

    let rows = stmt.query_map(params![block_id.to_string()], row_to_task)?;

    let mut tasks = Vec::new();
    for row in rows {
        tasks.push(row?);
    }

    Ok(tasks)
}

pub fn get_all_blocks_by_user_id(conn: &Connection, user_id: Uuid) -> Result<Vec<Block>> {
    let mut stmt = conn.prepare(
        "SELECT block_id, user_id, start_time, end_time FROM blocks WHERE user_id = ?1 ORDER BY start_time ASC",
    )?;
    let rows = stmt.query_map(params![user_id.to_string()], row_to_block)?;

    let mut blocks = Vec::new();
    for row in rows {
        blocks.push(row?);
    }

    Ok(blocks)
}

pub fn delete_block(conn: &Connection, block_id: Uuid) -> Result<usize> {
    conn.execute(
        "DELETE FROM blocks WHERE block_id = ?1",
        params![block_id.to_string()],
    )
}
// Pomodoro CREATE
pub fn create_pomodoro(conn: &Connection, pomodoro: &Pomodoro) -> Result<()> {
    conn.execute(
        "INSERT INTO pomodoros (pomodoro_id, user_id, pomodoro_name, pomodoro_cover, work_duration, short_break_duration, long_break_duration, long_break_interval, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![&pomodoro.pomodoro_id.to_string(), &pomodoro.user_id.to_string(), &pomodoro.pomodoro_name, &pomodoro.pomodoro_cover, &pomodoro.work_duration, &pomodoro.short_break_duration, &pomodoro.long_break_duration, &pomodoro.long_break_interval, &pomodoro.created_at.to_rfc3339(), &pomodoro.updated_at.map(|dt| dt.to_rfc3339())],
    )?;
    Ok(())
}

// Pomodoro READ (by user_id)
pub fn get_pomodoros_by_user_id(conn: &Connection, user_id: Uuid) -> Result<Vec<Pomodoro>> {
    let mut stmt = conn.prepare("SELECT pomodoro_id, user_id, pomodoro_name, pomodoro_cover, work_duration, short_break_duration, long_break_duration, long_break_interval, created_at, updated_at FROM pomodoros WHERE user_id = ?1")?;
    let rows = stmt.query_map(params![user_id.to_string()], row_to_pomodoro)?;

    let mut pomodoros = Vec::new();
    for row in rows {
        pomodoros.push(row?);
    }

    Ok(pomodoros)
}

// PomodoroSession CREATE
pub fn create_pomodoro_session(conn: &Connection, session: &PomodoroSession) -> Result<()> {
    conn.execute(
        "INSERT INTO pomodoro_sessions (session_id, user_id, pomodoro_id, session_type, duration_seconds, completed, started_at, completed_at, notes) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            &session.session_id.to_string(),
            &session.user_id.to_string(),
            &session.pomodoro_id.map(|id| id.to_string()),
            &session.session_type,
            &session.duration_seconds,
            &session.completed,
            &session.started_at.to_rfc3339(),
            &session.completed_at.to_rfc3339(),
            &session.notes,
        ],
    )?;
    Ok(())
}

// PomodoroSession READ (by user_id with date range)
pub fn get_pomodoro_sessions_by_user_date_range(
    conn: &Connection,
    user_id: Uuid,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> Result<Vec<PomodoroSession>> {
    let mut stmt = conn.prepare(
        "SELECT session_id, user_id, pomodoro_id, session_type, duration_seconds, completed, started_at, completed_at, notes
         FROM pomodoro_sessions
         WHERE user_id = ?1 AND started_at >= ?2 AND started_at <= ?3
         ORDER BY started_at DESC"
    )?;
    let rows = stmt.query_map(
        params![
            user_id.to_string(),
            start_date.to_rfc3339(),
            end_date.to_rfc3339()
        ],
        row_to_pomodoro_session,
    )?;

    let mut sessions = Vec::new();
    for row in rows {
        sessions.push(row?);
    }

    Ok(sessions)
}

// PomodoroSession READ (recent sessions)
pub fn get_recent_pomodoro_sessions(
    conn: &Connection,
    user_id: Uuid,
    limit: i32,
) -> Result<Vec<PomodoroSession>> {
    let mut stmt = conn.prepare(
        "SELECT session_id, user_id, pomodoro_id, session_type, duration_seconds, completed, started_at, completed_at, notes
         FROM pomodoro_sessions
         WHERE user_id = ?1
         ORDER BY started_at DESC
         LIMIT ?2"
    )?;
    let rows = stmt.query_map(params![user_id.to_string(), limit], row_to_pomodoro_session)?;

    let mut sessions = Vec::new();
    for row in rows {
        sessions.push(row?);
    }

    Ok(sessions)
}

// PomodoroSession STATISTICS (by user_id)
pub fn get_pomodoro_stats_for_user(
    conn: &Connection,
    user_id: Uuid,
    days: i32,
) -> Result<(i32, i32, i32)> {
    // Returns (total_sessions, total_work_time_seconds, completed_sessions)
    let since = Utc::now() - chrono::Duration::days(days as i64);

    let mut stmt = conn.prepare(
        "SELECT COUNT(*),
                SUM(CASE WHEN session_type = 'work' THEN duration_seconds ELSE 0 END),
                SUM(CASE WHEN completed = TRUE THEN 1 ELSE 0 END)
         FROM pomodoro_sessions
         WHERE user_id = ?1 AND started_at >= ?2",
    )?;

    stmt.query_row(params![user_id.to_string(), since.to_rfc3339()], |row| {
        Ok((
            row.get(0).unwrap_or(0),
            row.get(1).unwrap_or(0),
            row.get(2).unwrap_or(0),
        ))
    })
}

// BlockedItem CREATE
pub fn create_blocked_item(conn: &Connection, blocked_item: &BlockedItem) -> Result<()> {
    conn.execute(
        "INSERT INTO blocked_items (item_id, user_id, item_type, identifier, is_active) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![&blocked_item.item_id.to_string(), &blocked_item.user_id.to_string(), &blocked_item.item_type, &blocked_item.identifier, &blocked_item.is_active],
    )?;
    Ok(())
}

// BlockedItem READ (by user_id, active)
pub fn get_active_blocked_items_by_user_id(
    conn: &Connection,
    user_id: Uuid,
) -> Result<Vec<BlockedItem>> {
    let mut stmt = conn.prepare("SELECT item_id, user_id, item_type, identifier, is_active FROM blocked_items WHERE user_id = ?1 AND is_active = TRUE")?;
    let rows = stmt.query_map(params![user_id.to_string()], row_to_blocked_item)?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }

    Ok(items)
}

// BlockedItem READ (by item_id)
pub fn get_blocked_item(conn: &Connection, item_id: Uuid) -> Result<Option<BlockedItem>> {
    let mut stmt = conn.prepare("SELECT item_id, user_id, item_type, identifier, is_active FROM blocked_items WHERE item_id = ?1")?;
    let mut rows = stmt.query_map(params![item_id.to_string()], row_to_blocked_item)?;
    rows.next().transpose()
}

// BlockedItem READ (by user_id, all)
pub fn get_all_blocked_items_by_user_id(
    conn: &Connection,
    user_id: Uuid,
) -> Result<Vec<BlockedItem>> {
    let mut stmt = conn.prepare("SELECT item_id, user_id, item_type, identifier, is_active FROM blocked_items WHERE user_id = ?1 ORDER BY is_active DESC, identifier ASC")?;
    let rows = stmt.query_map(params![user_id.to_string()], row_to_blocked_item)?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }

    Ok(items)
}

// BlockedItem UPDATE (status)
pub fn update_blocked_item_status(
    conn: &Connection,
    item_id: Uuid,
    is_active: bool,
) -> Result<usize> {
    conn.execute(
        "UPDATE blocked_items SET is_active = ?1 WHERE item_id = ?2",
        params![is_active, item_id.to_string()],
    )
}

// BlockedItem DELETE
pub fn delete_blocked_item(conn: &Connection, item_id: Uuid) -> Result<usize> {
    conn.execute(
        "DELETE FROM blocked_items WHERE item_id = ?1",
        params![item_id.to_string()],
    )
}
fn row_to_oplog_entry(row: &Row) -> rusqlite::Result<OplogEntry> {
    let data_raw: String = row.get(5)?;
    let data = serde_json::from_str(&data_raw).map_err(|e| conversion_failure(5, e))?;

    Ok(OplogEntry {
        id: parse_uuid_column(row, 0)?,
        device_id: parse_uuid_column(row, 1)?,
        timestamp: row.get(2)?,
        table: row.get(3)?,
        op_type: row.get(4)?,
        data,
    })
}

fn row_to_peer(row: &Row) -> rusqlite::Result<Peer> {
    Ok(Peer {
        peer_id: parse_uuid_column(row, 0)?,
        user_id: parse_uuid_column(row, 1)?,
        device_id: parse_uuid_column(row, 2)?,
        last_known_ip: row.get(3)?,
        last_sync_time: row.get(4)?,
    })
}

// OplogEntry CREATE
pub fn create_oplog_entry(conn: &Connection, entry: &OplogEntry) -> Result<()> {
    let data = serde_json::to_string(&entry.data).map_err(|e| conversion_failure(5, e))?;

    conn.execute(
        "INSERT INTO oplog (id, device_id, timestamp, table_name, op_type, data) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            &entry.id.to_string(),
            &entry.device_id.to_string(),
            entry.timestamp,
            &entry.table,
            &entry.op_type,
            &data,
        ],
    )?;
    Ok(())
}

// OplogEntry READ (get all since a certain timestamp)
pub fn get_oplog_entries_since(conn: &Connection, since: i64) -> Result<Vec<OplogEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, device_id, timestamp, table_name, op_type, data FROM oplog WHERE timestamp > ?1 ORDER BY timestamp ASC",
    )?;
    let rows = stmt.query_map(params![since], row_to_oplog_entry)?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }

    Ok(entries)
}

// Peer CREATE
pub fn create_peer(conn: &Connection, peer: &Peer) -> Result<()> {
    conn.execute(
        "INSERT INTO peers (peer_id, user_id, device_id, last_known_ip, last_sync_time) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            &peer.peer_id.to_string(),
            &peer.user_id.to_string(),
            &peer.device_id.to_string(),
            &peer.last_known_ip,
            &peer.last_sync_time
        ],
    )?;
    Ok(())
}

// Peer READ (by user_id)
pub fn get_peers_by_user_id(conn: &Connection, user_id: Uuid) -> Result<Vec<Peer>> {
    let mut stmt = conn.prepare("SELECT peer_id, user_id, device_id, last_known_ip, last_sync_time FROM peers WHERE user_id = ?1")?;
    let rows = stmt.query_map(params![user_id.to_string()], row_to_peer)?;

    let mut peers = Vec::new();
    for row in rows {
        peers.push(row?);
    }

    Ok(peers)
}

// Peer READ (by peer_id)
pub fn get_peer(conn: &Connection, peer_id: Uuid) -> Result<Peer> {
    let mut stmt = conn.prepare("SELECT peer_id, user_id, device_id, last_known_ip, last_sync_time FROM peers WHERE peer_id = ?1")?;
    let peer = stmt.query_row(params![peer_id.to_string()], row_to_peer)?;
    Ok(peer)
}

// Peer READ (all)
pub fn get_all_peers(conn: &Connection) -> Result<Vec<Peer>> {
    let mut stmt = conn.prepare("SELECT peer_id, user_id, device_id, last_known_ip, last_sync_time FROM peers")?;
    let rows = stmt.query_map(params![], row_to_peer)?;

    let mut peers = Vec::new();
    for row in rows {
        peers.push(row?);
    }

    Ok(peers)
}

// TaskList READ (by user)
pub fn get_task_lists_by_user_id(conn: &Connection, user_id: Uuid) -> Result<Vec<TaskList>> {
    let mut stmt =
        conn.prepare("SELECT list_id, user_id, name FROM task_lists WHERE user_id = ?1")?;
    let rows = stmt.query_map(params![user_id.to_string()], row_to_task_list)?;

    let mut task_lists = Vec::new();
    for row in rows {
        task_lists.push(row?);
    }

    Ok(task_lists)
}

// Sound CREATE
pub fn create_sound(conn: &Connection, sound: &Sound) -> Result<()> {
    conn.execute(
        "INSERT INTO sounds (sound_id, name, category, file_url) VALUES (?1, ?2, ?3, ?4)",
        params![
            &sound.sound_id.to_string(),
            &sound.name,
            &sound.category,
            &sound.file_url
        ],
    )?;
    Ok(())
}

// Sound READ (by sound_id)
pub fn get_sound(conn: &Connection, sound_id: Uuid) -> Result<Option<Sound>> {
    let mut stmt =
        conn.prepare("SELECT sound_id, name, category, file_url FROM sounds WHERE sound_id = ?1")?;
    let mut rows = stmt.query_map(params![sound_id.to_string()], row_to_sound)?;
    rows.next().transpose()
}

// Sound READ (all)
pub fn get_all_sounds(conn: &Connection) -> Result<Vec<Sound>> {
    let mut stmt = conn.prepare("SELECT sound_id, name, category, file_url FROM sounds")?;
    let rows = stmt.query_map([], row_to_sound)?;

    let mut sounds = Vec::new();
    for row in rows {
        sounds.push(row?);
    }

    Ok(sounds)
}

// Sound READ (by category)
pub fn get_sounds_by_category(conn: &Connection, category: &str) -> Result<Vec<Sound>> {
    let mut stmt =
        conn.prepare("SELECT sound_id, name, category, file_url FROM sounds WHERE category = ?1")?;
    let rows = stmt.query_map(params![category], row_to_sound)?;

    let mut sounds = Vec::new();
    for row in rows {
        sounds.push(row?);
    }

    Ok(sounds)
}

// FavoriteSound CREATE
pub fn create_favorite_sound(conn: &Connection, favorite_sound: &FavoriteSound) -> Result<()> {
    conn.execute(
        "INSERT INTO favorite_sounds (user_id, sound_id) VALUES (?1, ?2)",
        params![
            &favorite_sound.user_id.to_string(),
            &favorite_sound.sound_id.to_string()
        ],
    )?;
    Ok(())
}

// FavoriteSound READ (by user_id)
pub fn get_favorite_sounds_by_user_id(conn: &Connection, user_id: Uuid) -> Result<Vec<Sound>> {
    let mut stmt = conn.prepare(
        "
        SELECT s.sound_id, s.name, s.category, s.file_url
        FROM sounds s
        INNER JOIN favorite_sounds fs ON s.sound_id = fs.sound_id
        WHERE fs.user_id = ?1
    ",
    )?;
    let rows = stmt.query_map(params![user_id.to_string()], row_to_sound)?;

    let mut sounds = Vec::new();
    for row in rows {
        sounds.push(row?);
    }

    Ok(sounds)
}

// FavoriteSound DELETE
pub fn delete_favorite_sound(conn: &Connection, user_id: Uuid, sound_id: Uuid) -> Result<usize> {
    conn.execute(
        "DELETE FROM favorite_sounds WHERE user_id = ?1 AND sound_id = ?2",
        params![user_id.to_string(), sound_id.to_string()],
    )
}

// UserPreference row mapper
fn row_to_user_preference(row: &Row) -> rusqlite::Result<UserPreference> {
    Ok(UserPreference {
        preference_id: parse_uuid_column(row, 0)?,
        user_id: parse_uuid_column(row, 1)?,
        preference_key: row.get(2)?,
        preference_value: row.get(3)?,
        preference_type: row.get(4)?,
        created_at: parse_datetime_column(row, 5)?,
        updated_at: parse_optional_datetime_column(row, 6)?,
    })
}

// UserPreference CREATE
pub fn create_user_preference(conn: &Connection, pref: &UserPreference) -> Result<()> {
    conn.execute(
        "INSERT INTO user_preferences (preference_id, user_id, preference_key, preference_value, preference_type, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            &pref.preference_id.to_string(),
            &pref.user_id.to_string(),
            &pref.preference_key,
            &pref.preference_value,
            &pref.preference_type,
            &pref.created_at.to_rfc3339(),
            &pref.updated_at.map(|dt| dt.to_rfc3339()),
        ],
    )?;
    Ok(())
}

// UserPreference READ (by user_id and key)
pub fn get_user_preference(
    conn: &Connection,
    user_id: Uuid,
    preference_key: &str,
) -> Result<Option<UserPreference>> {
    let mut stmt = conn.prepare(
        "SELECT preference_id, user_id, preference_key, preference_value, preference_type, created_at, updated_at
         FROM user_preferences
         WHERE user_id = ?1 AND preference_key = ?2"
    )?;
    let mut rows = stmt.query_map(
        params![user_id.to_string(), preference_key],
        row_to_user_preference,
    )?;
    rows.next().transpose()
}

// UserPreference READ (all by user_id)
pub fn get_all_user_preferences(conn: &Connection, user_id: Uuid) -> Result<Vec<UserPreference>> {
    let mut stmt = conn.prepare(
        "SELECT preference_id, user_id, preference_key, preference_value, preference_type, created_at, updated_at
         FROM user_preferences
         WHERE user_id = ?1
         ORDER BY preference_key ASC"
    )?;
    let rows = stmt.query_map(params![user_id.to_string()], row_to_user_preference)?;

    let mut prefs = Vec::new();
    for row in rows {
        prefs.push(row?);
    }

    Ok(prefs)
}

// UserPreference UPDATE (upsert: insert or update)
pub fn upsert_user_preference(conn: &Connection, pref: &UserPreference) -> Result<()> {
    conn.execute(
        "INSERT INTO user_preferences (preference_id, user_id, preference_key, preference_value, preference_type, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(user_id, preference_key)
         DO UPDATE SET
            preference_value = excluded.preference_value,
            preference_type = excluded.preference_type,
            updated_at = excluded.updated_at",
        params![
            &pref.preference_id.to_string(),
            &pref.user_id.to_string(),
            &pref.preference_key,
            &pref.preference_value,
            &pref.preference_type,
            &pref.created_at.to_rfc3339(),
            &pref.updated_at.map(|dt| dt.to_rfc3339()),
        ],
    )?;
    Ok(())
}

// UserPreference DELETE
pub fn delete_user_preference(
    conn: &Connection,
    user_id: Uuid,
    preference_key: &str,
) -> Result<usize> {
    conn.execute(
        "DELETE FROM user_preferences WHERE user_id = ?1 AND preference_key = ?2",
        params![user_id.to_string(), preference_key],
    )
}
