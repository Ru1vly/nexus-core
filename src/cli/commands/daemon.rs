use crate::cli::config::Config;
use crate::cli::daemon as daemon_utils;
use crate::cli::errors::{CliError, CliResult};
use crate::cli::output;
use crate::db::operations::initialize_database;
use crate::logic::sync::{create_swarm, P2PConfig};
use crate::logic::sync_manager::SyncManager;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time;

pub async fn start(
    daemon: bool,
    port: u16,
    config_path: Option<&str>,
    config: &Config,
) -> CliResult<()> {
    let pid_file = Config::pid_file();

    // Check if already running
    if daemon_utils::is_running(&pid_file) {
        return Err(CliError::DaemonError(
            "Daemon is already running".to_string(),
        ));
    }

    // Check if user is configured
    let user_config = config
        .user
        .as_ref()
        .ok_or_else(|| CliError::ConfigError("User not configured. Run 'nexus-cli init' first.".to_string()))?;

    let device_config = config
        .device
        .as_ref()
        .ok_or_else(|| CliError::ConfigError("Device not configured. Run 'nexus-cli init' first.".to_string()))?;

    if daemon {
        output::step("Starting daemon in background mode");

        #[cfg(unix)]
        {
            daemon_utils::daemonize(&pid_file)?;
            // After daemonize, we're in the child process
            run_sync_loop(port, config).await?;
        }

        #[cfg(not(unix))]
        {
            return Err(CliError::DaemonError(
                "Daemon mode is only supported on Unix-like systems. Run in foreground mode.".to_string(),
            ));
        }
    } else {
        output::step("Starting sync in foreground mode");

        // Write PID file
        let pid = std::process::id() as i32;
        daemon_utils::write_pid(&pid_file, pid)?;

        // Setup Ctrl+C handler
        let pid_file_clone = pid_file.clone();
        ctrlc::set_handler(move || {
            output::info("Received shutdown signal, cleaning up...");
            daemon_utils::remove_pid_file(&pid_file_clone).ok();
            std::process::exit(0);
        })
        .map_err(|e| CliError::DaemonError(format!("Failed to set Ctrl+C handler: {}", e)))?;

        output::success("Sync started (press Ctrl+C to stop)");

        // Run sync loop
        run_sync_loop(port, config).await?;
    }

    Ok(())
}

async fn run_sync_loop(port: u16, config: &Config) -> CliResult<()> {
    let user_config = config.user.as_ref().unwrap();
    let device_config = config.device.as_ref().unwrap();

    // Parse UUIDs
    let user_id = uuid::Uuid::parse_str(&user_config.id)
        .map_err(|_| CliError::ConfigError("Invalid user ID".to_string()))?;
    let device_id = uuid::Uuid::parse_str(&device_config.id)
        .map_err(|_| CliError::ConfigError("Invalid device ID".to_string()))?;

    // Initialize database connection
    let db_path = config.db_path();
    let conn = initialize_database(&db_path)
        .map_err(|e| CliError::DatabaseError(e.to_string()))?;
    let conn = Arc::new(Mutex::new(conn));

    // Generate keypair for P2P
    let (peer_id, keypair) = crate::logic::sync::generate_device_id();
    log::info!("Generated peer ID: {}", peer_id);

    // Create P2P config
    let p2p_config = P2PConfig {
        enable_mdns: config.sync.enable_mdns,
        enable_relay: config.sync.enable_relay,
        bootstrap_nodes: config.network.bootstrap_nodes.clone(),
        relay_servers: config.network.relay_servers.clone(),
        heartbeat_interval: Duration::from_secs(config.sync.heartbeat_interval_secs),
        max_message_size: config.sync.max_message_size,
    };

    // Create sync manager
    let mut sync_manager = SyncManager::new(keypair, user_id, device_id, conn.clone(), p2p_config)
        .map_err(|e| CliError::SyncError(format!("Failed to create sync manager: {}", e)))?;

    // Start listening
    let listen_addr = format!("/ip4/{}/tcp/{}", config.network.listen_address, port);
    sync_manager
        .listen(port)
        .map_err(|e| CliError::SyncError(format!("Failed to start listening: {}", e)))?;

    log::info!("Listening on {}", listen_addr);

    // Connect to bootstrap nodes and relay servers
    if !config.network.bootstrap_nodes.is_empty() || !config.network.relay_servers.is_empty() {
        sync_manager
            .connect_to_network(&config.network.bootstrap_nodes, &config.network.relay_servers)
            .map_err(|e| CliError::SyncError(format!("Failed to connect to network: {}", e)))?;
    }

    // Announce presence
    sync_manager
        .announce_presence()
        .map_err(|e| CliError::SyncError(format!("Failed to announce presence: {}", e)))?;

    log::info!("Sync manager initialized and running");

    // Main event loop
    loop {
        // Process events
        if let Err(e) = sync_manager.process_event().await {
            log::error!("Error processing event: {}", e);
        }

        // Small delay to prevent busy loop
        time::sleep(Duration::from_millis(100)).await;
    }
}

pub async fn stop(config: &Config) -> CliResult<()> {
    let pid_file = Config::pid_file();

    output::step("Stopping daemon");

    daemon_utils::stop_daemon(&pid_file)?;

    output::success("Daemon stopped");

    Ok(())
}

pub async fn restart(daemon: bool, config: &Config) -> CliResult<()> {
    let pid_file = Config::pid_file();

    // Stop if running
    if daemon_utils::is_running(&pid_file) {
        output::step("Stopping existing daemon");
        daemon_utils::stop_daemon(&pid_file)?;
        output::success("Daemon stopped");
    }

    // Wait a moment
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Start again
    start(daemon, 0, None, config).await?;

    Ok(())
}

pub async fn status(watch: bool, interval: u64, json: bool, config: &Config) -> CliResult<()> {
    let pid_file = Config::pid_file();

    if watch {
        // Watch mode - continuous monitoring
        loop {
            print_status(json, config, &pid_file)?;

            if !json {
                // Clear screen on next iteration
                print!("\x1B[2J\x1B[1;1H");
            }

            tokio::time::sleep(Duration::from_secs(interval)).await;
        }
    } else {
        // Single status check
        print_status(json, config, &pid_file)?;
    }

    Ok(())
}

fn print_status(json: bool, config: &Config, pid_file: &std::path::Path) -> CliResult<()> {
    let is_running = daemon_utils::is_running(pid_file);

    if json {
        let status = serde_json::json!({
            "status": if is_running { "running" } else { "stopped" },
            "pid": if is_running { daemon_utils::get_pid(pid_file).ok() } else { None },
            "uptime_seconds": if is_running { daemon_utils::get_uptime(pid_file).ok() } else { None },
            "database": config.db_path(),
            "sync_enabled": config.sync.enabled,
        });
        output::json(&status);
    } else {
        let status_text = if is_running { "Running" } else { "Stopped" };
        let uptime_text = if is_running {
            daemon_utils::get_uptime(pid_file)
                .map(|u| daemon_utils::format_uptime(u))
                .unwrap_or_else(|_| "Unknown".to_string())
        } else {
            "N/A".to_string()
        };

        let pid_text = if is_running {
            daemon_utils::get_pid(pid_file)
                .map(|p| p.to_string())
                .unwrap_or_else(|_| "Unknown".to_string())
        } else {
            "N/A".to_string()
        };

        output::print_box(
            "Nexus Sync Status",
            vec![
                ("Status", status_text),
                ("PID", &pid_text),
                ("Uptime", &uptime_text),
                ("Database", &config.db_path()),
                ("Sync Enabled", if config.sync.enabled { "Yes" } else { "No" }),
            ],
        );
    }

    Ok(())
}
