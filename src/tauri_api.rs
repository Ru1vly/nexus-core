//! Tauri API module - only available with the "tauri-api" feature
//!
//! This module provides Tauri command handlers for the ahenk library.

#[cfg(feature = "tauri-api")]
mod tauri_commands {
    use crate::db::operations::get_habit;
    use crate::logic::sync_manager::SyncManager;
    use crate::logic::{
        add_item_to_blocklist, add_task_to_list, assign_task_to_block, create_habit,
        create_new_task_list, create_soundscape_preset, delete_soundscape_preset,
        get_all_blocklist, get_all_habits_for_user, get_all_pomodoro_presets,
        get_all_task_lists_for_user, get_all_tasks_in_list, get_habit_streak,
        get_pomodoro_statistics, get_recent_sessions, get_soundscape_presets_for_user,
        get_tasks_due_today, get_tasks_for_a_specific_block, log_habit_completion, login_user,
        mark_task_as_complete, record_pomodoro_session, register_user, remove_blocked_item,
        save_pomodoro_preset, schedule_block, toggle_blocked_item, update_soundscape_preset,
    };
    use crate::models::{
        Block, BlockedItem, Habit, Pomodoro, PomodoroPreset, PomodoroSession, Task, TaskList, User,
    };
    use chrono::{DateTime, NaiveDate, Utc};
    use rusqlite::Connection;
    use std::sync::Arc;
    use std::sync::Mutex;
    use tauri::State;
    use uuid::Uuid;

    // TODO: Add AppState with persistent device_id
    // Currently each command generates a new device_id which breaks CRDT sync
    // Should be:
    // pub struct AppState {
    //     pub conn: Mutex<Connection>,
    //     pub device_id: Uuid,  // Persistent device ID for this app instance
    // }
    pub struct DbConnection(pub Mutex<Connection>);

    // ============= User Management =============
    #[tauri::command]
    pub fn nexus_register_user(
        username: String,
        email: String,
        password: String,
        conn: State<DbConnection>,
    ) -> Result<User, String> {
        let mut db = conn.0.lock().map_err(|e| e.to_string())?;
        register_user(&mut db, username, email, password).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_login_user(
        username: String,
        password: String,
        conn: State<DbConnection>,
    ) -> Result<User, String> {
        let db = conn.0.lock().map_err(|e| e.to_string())?;
        login_user(&db, &username, &password).map_err(|e| e.to_string())
    }

    // ============= Task Management =============
    #[tauri::command]
    pub fn nexus_create_task_list(
        user_id: String,
        name: String,
        conn: State<DbConnection>,
    ) -> Result<TaskList, String> {
        let mut db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        let device_id = Uuid::new_v4(); // Assuming device_id can be generated or passed from frontend
        create_new_task_list(&mut db, user_uuid, device_id, name).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_get_task_lists(
        user_id: String,
        conn: State<DbConnection>,
    ) -> Result<Vec<TaskList>, String> {
        let db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        get_all_task_lists_for_user(&db, user_uuid).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_add_task(
        user_id: String,
        list_id: String,
        content: String,
        due_date: Option<String>,
        conn: State<DbConnection>,
    ) -> Result<Task, String> {
        let mut db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        let device_id = Uuid::new_v4(); // Assuming device_id can be generated or passed from frontend
        let list_uuid = Uuid::parse_str(&list_id).map_err(|e| e.to_string())?;
        let parsed_due_date = due_date
            .map(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d"))
            .transpose()
            .map_err(|e| e.to_string())?;
        add_task_to_list(
            &mut db,
            user_uuid,
            device_id,
            list_uuid,
            content,
            parsed_due_date,
        )
        .map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_get_tasks(
        user_id: String,
        list_id: String,
        conn: State<DbConnection>,
    ) -> Result<Vec<Task>, String> {
        let db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        let list_uuid = Uuid::parse_str(&list_id).map_err(|e| e.to_string())?;
        get_all_tasks_in_list(&db, user_uuid, list_uuid).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_mark_task_complete(
        user_id: String,
        task_id: String,
        conn: State<DbConnection>,
    ) -> Result<(), String> {
        let mut db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        let device_id = Uuid::new_v4(); // Assuming device_id can be generated or passed from frontend
        let task_uuid = Uuid::parse_str(&task_id).map_err(|e| e.to_string())?;
        mark_task_as_complete(&mut db, user_uuid, device_id, task_uuid).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_get_tasks_due_today(
        user_id: String,
        conn: State<DbConnection>,
    ) -> Result<Vec<Task>, String> {
        let db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        get_tasks_due_today(&db, user_uuid).map_err(|e| e.to_string())
    }

    // ============= Habit Management =============
    #[tauri::command]
    pub fn nexus_create_habit(
        user_id: String,
        name: String,
        description: Option<String>,
        habit_cover: Option<String>,
        frequency_type: String,
        conn: State<DbConnection>,
    ) -> Result<Habit, String> {
        let mut db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        let device_id = Uuid::new_v4(); // Assuming device_id can be generated or passed from frontend
        create_habit(
            &mut db,
            user_uuid,
            device_id,
            name,
            description,
            habit_cover,
            frequency_type,
        )
        .map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_get_habits(
        user_id: String,
        conn: State<DbConnection>,
    ) -> Result<Vec<Habit>, String> {
        let db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        get_all_habits_for_user(&db, user_uuid).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_log_habit_completion(
        user_id: String,
        habit_id: String,
        date: String,
        notes: Option<String>,
        conn: State<DbConnection>,
    ) -> Result<(), String> {
        let mut db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        let device_id = Uuid::new_v4(); // Assuming device_id can be generated or passed from frontend
        let habit_uuid = Uuid::parse_str(&habit_id).map_err(|e| e.to_string())?;
        let parsed_date =
            NaiveDate::parse_from_str(&date, "%Y-%m-%d").map_err(|e| e.to_string())?;
        log_habit_completion(
            &mut db,
            user_uuid,
            device_id,
            habit_uuid,
            parsed_date,
            notes,
        )
        .map_err(|e| e.to_string())
        .map(|_| ()) // Map HabitEntry to ()
    }

    #[tauri::command]
    pub fn nexus_get_habit_streak(
        user_id: String,
        habit_id: String,
        conn: State<DbConnection>,
    ) -> Result<i32, String> {
        let db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        let habit_uuid = Uuid::parse_str(&habit_id).map_err(|e| e.to_string())?;
        get_habit_streak(&db, user_uuid, habit_uuid).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_get_habit(habit_id: String, conn: State<DbConnection>) -> Result<Habit, String> {
        let db = conn.0.lock().map_err(|e| e.to_string())?;
        let habit_uuid = Uuid::parse_str(&habit_id).map_err(|e| e.to_string())?;
        get_habit(&db, habit_uuid)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Habit not found".to_string())
    }

    // ============= Pomodoro/Timer Management =============
    #[tauri::command]
    pub fn nexus_save_pomodoro_preset(
        user_id: String,
        name: String,
        cover: Option<String>,
        work_duration: i32,
        short_break: i32,
        long_break: i32,
        interval: i32,
        conn: State<DbConnection>,
    ) -> Result<(), String> {
        let mut db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        let device_id = Uuid::new_v4(); // Assuming device_id can be generated or passed from frontend
        let preset = PomodoroPreset {
            name,
            cover,
            work_duration,
            short_break,
            long_break,
            interval,
        };
        save_pomodoro_preset(&mut db, user_uuid, device_id, preset)
            .map_err(|e| e.to_string())
            .map(|_| ()) // Map Pomodoro to ()
    }

    #[tauri::command]
    pub fn nexus_get_pomodoro_presets(
        user_id: String,
        conn: State<DbConnection>,
    ) -> Result<Vec<Pomodoro>, String> {
        // Changed return type to Vec<Pomodoro>
        let db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        get_all_pomodoro_presets(&db, user_uuid).map_err(|e| e.to_string())
    }

    // ============= Time Blocking =============
    #[tauri::command]
    pub fn nexus_schedule_block(
        user_id: String,
        start_time: String,
        end_time: String,
        conn: State<DbConnection>,
    ) -> Result<Block, String> {
        let mut db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        let device_id = Uuid::new_v4(); // Assuming device_id can be generated or passed from frontend
        let start = DateTime::parse_from_rfc3339(&start_time)
            .map_err(|e| e.to_string())?
            .with_timezone(&Utc);
        let end = DateTime::parse_from_rfc3339(&end_time)
            .map_err(|e| e.to_string())?
            .with_timezone(&Utc);
        schedule_block(&mut db, user_uuid, device_id, start, end).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_assign_task_to_block(
        user_id: String,
        task_id: String,
        block_id: String,
        conn: State<DbConnection>,
    ) -> Result<(), String> {
        let mut db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        let device_id = Uuid::new_v4(); // Assuming device_id can be generated or passed from frontend
        let task_uuid = Uuid::parse_str(&task_id).map_err(|e| e.to_string())?;
        let block_uuid = Uuid::parse_str(&block_id).map_err(|e| e.to_string())?;
        assign_task_to_block(&mut db, user_uuid, device_id, task_uuid, block_uuid)
            .map_err(|e| e.to_string())
            .map(|_| ()) // Map TaskBlock to ()
    }

    #[tauri::command]
    pub fn nexus_get_tasks_for_block(
        user_id: String,
        block_id: String,
        conn: State<DbConnection>,
    ) -> Result<Vec<Task>, String> {
        let db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        let block_uuid = Uuid::parse_str(&block_id).map_err(|e| e.to_string())?;
        get_tasks_for_a_specific_block(&db, user_uuid, block_uuid).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_get_all_blocks(
        user_id: String,
        conn: State<DbConnection>,
    ) -> Result<Vec<Block>, String> {
        let db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        get_all_blocks_for_user(&db, user_uuid).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_remove_block(
        user_id: String,
        block_id: String,
        conn: State<DbConnection>,
    ) -> Result<(), String> {
        let mut db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        let device_id = Uuid::new_v4(); // Assuming device_id can be generated or passed from frontend
        let block_uuid = Uuid::parse_str(&block_id).map_err(|e| e.to_string())?;
        remove_block(&mut db, user_uuid, device_id, block_uuid).map_err(|e| e.to_string())
    }

    // ============= Blocker Management =============
    #[tauri::command]
    pub fn nexus_add_blocked_item(
        user_id: String,
        item_type: String,
        identifier: String,
        conn: State<DbConnection>,
    ) -> Result<BlockedItem, String> {
        let mut db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        let device_id = Uuid::new_v4(); // Assuming device_id can be generated or passed from frontend
        add_item_to_blocklist(&mut db, user_uuid, device_id, item_type, identifier)
            .map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_get_all_blocked_items(
        user_id: String,
        conn: State<DbConnection>,
    ) -> Result<Vec<BlockedItem>, String> {
        let db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        get_all_blocklist(&db, user_uuid).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_toggle_blocked_item(
        user_id: String,
        item_id: String,
        is_active: bool,
        conn: State<DbConnection>,
    ) -> Result<(), String> {
        let mut db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        let device_id = Uuid::new_v4(); // Assuming device_id can be generated or passed from frontend
        let item_uuid = Uuid::parse_str(&item_id).map_err(|e| e.to_string())?;
        toggle_blocked_item(&mut db, user_uuid, device_id, item_uuid, is_active)
            .map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_remove_blocked_item(
        user_id: String,
        item_id: String,
        conn: State<DbConnection>,
    ) -> Result<(), String> {
        let mut db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        let device_id = Uuid::new_v4(); // Assuming device_id can be generated or passed from frontend
        let item_uuid = Uuid::parse_str(&item_id).map_err(|e| e.to_string())?;
        remove_blocked_item(&mut db, user_uuid, device_id, item_uuid).map_err(|e| e.to_string())
    }

    // ============= Pomodoro Session Tracking =============
    #[tauri::command]
    pub fn nexus_record_session(
        user_id: String,
        pomodoro_id: Option<String>,
        session_type: String,
        duration_seconds: i32,
        completed: bool,
        started_at: String,
        notes: Option<String>,
        conn: State<DbConnection>,
    ) -> Result<PomodoroSession, String> {
        let mut db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        let device_id = Uuid::new_v4();
        let pomodoro_uuid = pomodoro_id
            .map(|id| Uuid::parse_str(&id))
            .transpose()
            .map_err(|e| e.to_string())?;
        let start_time = DateTime::parse_from_rfc3339(&started_at)
            .map_err(|e| e.to_string())?
            .with_timezone(&Utc);

        record_pomodoro_session(
            &mut db,
            user_uuid,
            device_id,
            pomodoro_uuid,
            session_type,
            duration_seconds,
            completed,
            start_time,
            notes,
        )
        .map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_get_recent_sessions(
        user_id: String,
        limit: i32,
        conn: State<DbConnection>,
    ) -> Result<Vec<PomodoroSession>, String> {
        let db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        get_recent_sessions(&db, user_uuid, limit).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_get_pomodoro_stats(
        user_id: String,
        days: i32,
        conn: State<DbConnection>,
    ) -> Result<(i32, i32, i32), String> {
        let db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        get_pomodoro_statistics(&db, user_uuid, days).map_err(|e| e.to_string())
    }

    // ============= Soundscape Management =============
    #[tauri::command]
    pub fn nexus_create_soundscape(
        user_id: String,
        name: String,
        file_path: String,
        volume: f32,
        is_playing: bool,
        conn: State<DbConnection>,
    ) -> Result<crate::models::Soundscape, String> {
        let mut db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        let device_id = Uuid::new_v4(); // Assuming device_id can be generated or passed from frontend
        crate::logic::create_soundscape(
            &mut db, user_uuid, device_id, name, file_path, volume, is_playing,
        )
        .map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_get_all_soundscapes(
        user_id: String,
        conn: State<DbConnection>,
    ) -> Result<Vec<crate::models::Soundscape>, String> {
        let db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        crate::logic::get_all_soundscapes_for_user(&db, user_uuid).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_update_soundscape(
        soundscape_id: String,
        name: String,
        file_path: String,
        volume: f32,
        is_playing: bool,
        conn: State<DbConnection>,
    ) -> Result<(), String> {
        let mut db = conn.0.lock().map_err(|e| e.to_string())?;
        let soundscape_uuid = Uuid::parse_str(&soundscape_id).map_err(|e| e.to_string())?;
        let device_id = Uuid::new_v4(); // Assuming device_id can be generated or passed from frontend
        crate::logic::update_soundscape(
            &mut db,
            soundscape_uuid,
            device_id,
            name,
            file_path,
            volume,
            is_playing,
        )
        .map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_delete_soundscape(
        soundscape_id: String,
        conn: State<DbConnection>,
    ) -> Result<(), String> {
        let mut db = conn.0.lock().map_err(|e| e.to_string())?;
        let soundscape_uuid = Uuid::parse_str(&soundscape_id).map_err(|e| e.to_string())?;
        let device_id = Uuid::new_v4(); // Assuming device_id can be generated or passed from frontend
        crate::logic::delete_soundscape(&mut db, device_id, soundscape_uuid)
            .map_err(|e| e.to_string())
    }

    // ============= Soundscape Preset Management =============
    #[tauri::command]
    pub fn nexus_get_soundscape_presets(
        user_id: String,
        conn: State<DbConnection>,
    ) -> Result<Vec<crate::models::SoundscapePreset>, String> {
        let db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        crate::logic::get_soundscape_presets_for_user(&db, user_uuid).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_create_soundscape_preset(
        user_id: String,
        name: String,
        tracks: Vec<crate::models::SoundTrackDto>,
        conn: State<DbConnection>,
    ) -> Result<crate::models::SoundscapePreset, String> {
        let mut db = conn.0.lock().map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        let device_id = Uuid::new_v4();
        crate::logic::create_soundscape_preset(&mut db, user_uuid, device_id, name, tracks)
            .map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_update_soundscape_preset(
        preset_id: String,
        name: String,
        tracks: Vec<crate::models::SoundTrackDto>,
        conn: State<DbConnection>,
    ) -> Result<crate::models::SoundscapePreset, String> {
        let mut db = conn.0.lock().map_err(|e| e.to_string())?;
        let preset_uuid = Uuid::parse_str(&preset_id).map_err(|e| e.to_string())?;
        let device_id = Uuid::new_v4();
        crate::logic::update_soundscape_preset(&mut db, preset_uuid, device_id, name, tracks)
            .map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_delete_soundscape_preset(
        preset_id: String,
        conn: State<DbConnection>,
    ) -> Result<(), String> {
        let mut db = conn.0.lock().map_err(|e| e.to_string())?;
        let preset_uuid = Uuid::parse_str(&preset_id).map_err(|e| e.to_string())?;
        let device_id = Uuid::new_v4();
        crate::logic::delete_soundscape_preset(&mut db, device_id, preset_uuid)
            .map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_get_sync_status(
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

    #[tauri::command]
    pub fn nexus_request_sync(
        user_id: String,
        sync_manager_state: tauri::State<Arc<Mutex<SyncManager>>>,
    ) -> Result<(), String> {
        let mut sync_manager = sync_manager_state
            .inner()
            .lock()
            .map_err(|e| e.to_string())?;
        let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
        // For now, request sync from the beginning of time if no last sync time is available
        let since = sync_manager.get_last_sync_time().unwrap_or_else(Utc::now);
        sync_manager.request_sync(since).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn nexus_set_online_status(
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

    #[tauri::command]
    pub fn nexus_sync_pending_changes(
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
