use crate::cli::config::Config;
use crate::cli::errors::{CliError, CliResult};
use crate::cli::output;
use std::process::Command;

pub async fn set(key: &str, value: &str, config: &Config) -> CliResult<()> {
    let mut config = config.clone();

    output::step(&format!("Setting {} = {}", key, value));

    config.set_value(key, value)?;
    config.save(None)?;

    output::success(&format!("Configuration updated: {} = {}", key, value));

    Ok(())
}

pub async fn get(key: &str, config: &Config) -> CliResult<()> {
    let value = config.get_value(key)?;
    println!("{}", value);
    Ok(())
}

pub async fn list(json: bool, config: &Config) -> CliResult<()> {
    if json {
        let config_json = serde_json::to_value(config)
            .map_err(|e| CliError::ConfigError(format!("Failed to serialize config: {}", e)))?;
        output::json(&config_json);
    } else {
        output::header("Current Configuration");

        println!();
        output::key_value("Database Path", &config.database.path);
        output::key_value("Auto Migrate", &config.database.auto_migrate.to_string());

        if let Some(user) = &config.user {
            println!();
            output::key_value("User ID", &user.id);
            output::key_value("User Name", &user.name);
            output::key_value("User Email", &user.email);
        }

        if let Some(device) = &config.device {
            println!();
            output::key_value("Device ID", &device.id);
            output::key_value("Device Type", &device.device_type);
            output::key_value("Device Name", &device.name);
        }

        println!();
        output::key_value("Sync Enabled", &config.sync.enabled.to_string());
        output::key_value("Auto Start", &config.sync.auto_start.to_string());
        output::key_value("Enable mDNS", &config.sync.enable_mdns.to_string());
        output::key_value("Enable Relay", &config.sync.enable_relay.to_string());

        println!();
        output::key_value("Listen Port", &config.network.listen_port.to_string());
        output::key_value("Listen Address", &config.network.listen_address);

        println!();
        output::key_value("Log Level", &config.logging.level);
        output::key_value("Log File", &config.logging.file);
    }

    Ok(())
}

pub async fn edit(config: &Config) -> CliResult<()> {
    let config_path = Config::default_path();

    // Make sure config file exists
    if !config_path.exists() {
        config.save(None)?;
    }

    // Get editor from environment or use default
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| {
        if cfg!(target_os = "windows") {
            "notepad".to_string()
        } else {
            "vi".to_string()
        }
    });

    output::step(&format!("Opening {} in {}", config_path.display(), editor));

    // Open editor
    let status = Command::new(&editor)
        .arg(&config_path)
        .status()
        .map_err(|e| CliError::IoError(e))?;

    if !status.success() {
        return Err(CliError::Other("Editor exited with error".to_string()));
    }

    output::success("Configuration file edited");
    output::info("Configuration will be reloaded on next command");

    Ok(())
}
