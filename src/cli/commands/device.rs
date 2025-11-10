use crate::cli::config::Config;
use crate::cli::errors::{CliError, CliResult};
use crate::cli::output;
use crate::db::operations::{get_devices_by_user_id, initialize_database};

pub async fn list(json: bool, config: &Config) -> CliResult<()> {
    let user_config = config
        .user
        .as_ref()
        .ok_or_else(|| CliError::ConfigError("User not configured".to_string()))?;

    let user_id = uuid::Uuid::parse_str(&user_config.id)
        .map_err(|_| CliError::ConfigError("Invalid user ID".to_string()))?;

    let db_path = config.db_path();
    let conn = initialize_database(&db_path)
        .map_err(|e| CliError::DatabaseError(e.to_string()))?;

    let devices = get_devices_by_user_id(&conn, user_id)
        .map_err(|e| CliError::DatabaseError(e.to_string()))?;

    if json {
        let devices_json: Vec<_> = devices
            .iter()
            .map(|d| {
                serde_json::json!({
                    "device_id": d.device_id.to_string(),
                    "device_type": d.device_type,
                    "last_seen": d.last_seen,
                })
            })
            .collect();
        output::json(&serde_json::json!(devices_json));
    } else {
        if devices.is_empty() {
            output::info("No devices found");
            return Ok(());
        }

        let mut table = output::create_table(vec!["Device ID", "Type", "Last Seen"]);

        for device in devices {
            table.add_row(prettytable::Row::new(vec![
                prettytable::Cell::new(&device.device_id.to_string()),
                prettytable::Cell::new(&device.device_type),
                prettytable::Cell::new(&device.last_seen.map(|t| t.to_string()).unwrap_or_else(|| "Never".to_string())),
            ]));
        }

        table.printstd();
    }

    Ok(())
}

pub async fn pair(device_type: &str, name: Option<&str>, config: &Config) -> CliResult<()> {
    output::step(&format!("Generating pairing QR code for {} device", device_type));

    // TODO: Implement device pairing with QR code generation
    output::warning("Device pairing not yet fully implemented");
    output::info("This feature requires the authentication challenge system to be integrated");

    Ok(())
}

pub async fn authorize(code: &str, config: &Config) -> CliResult<()> {
    output::step("Authorizing device with code");

    // TODO: Implement device authorization
    output::warning("Device authorization not yet fully implemented");

    Ok(())
}

pub async fn remove(device_id: &str, config: &Config) -> CliResult<()> {
    output::step(&format!("Removing device: {}", device_id));

    // TODO: Implement device removal
    output::warning("Device removal not yet implemented");

    Ok(())
}
