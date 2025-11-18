use crate::cli::config::Config;
use crate::cli::errors::{CliError, CliResult};
use crate::cli::output;
use crate::db::operations::{delete_device, get_devices_by_user_id, initialize_database};
use crate::{AuthResult, AuthorizerWorkflow, NewDeviceWorkflow};
use libp2p::identity::Keypair;

/// Detect the current device type based on platform
fn detect_device_type() -> String {
    #[cfg(target_os = "android")]
    return "phone".to_string();

    #[cfg(target_os = "ios")]
    return "phone".to_string();

    #[cfg(target_os = "macos")]
    return "desktop".to_string();

    #[cfg(target_os = "windows")]
    return "desktop".to_string();

    #[cfg(target_os = "linux")]
    return "desktop".to_string();

    #[cfg(not(any(
        target_os = "android",
        target_os = "ios",
        target_os = "macos",
        target_os = "windows",
        target_os = "linux"
    )))]
    return "unknown".to_string();
}

pub async fn list(json: bool, config: &Config) -> CliResult<()> {
    let user_config = config
        .user
        .as_ref()
        .ok_or_else(|| CliError::ConfigError("User not configured".to_string()))?;

    let user_id = uuid::Uuid::parse_str(&user_config.id)
        .map_err(|_| CliError::ConfigError("Invalid user ID".to_string()))?;

    let db_path = config.db_path();
    let conn = initialize_database(&db_path).map_err(|e| CliError::DatabaseError(e.to_string()))?;

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
                prettytable::Cell::new(
                    &device
                        .last_seen
                        .map(|t| t.to_string())
                        .unwrap_or_else(|| "Never".to_string()),
                ),
            ]));
        }

        table.printstd();
    }

    Ok(())
}

pub async fn pair(device_type: &str, name: Option<&str>, config: &Config) -> CliResult<()> {
    output::step(&format!(
        "Generating pairing QR code for {} device",
        device_type
    ));

    let user_config = config.user.as_ref().ok_or_else(|| {
        CliError::ConfigError("User not configured. Run 'ahenk-cli init' first".to_string())
    })?;

    let user_id = uuid::Uuid::parse_str(&user_config.id)
        .map_err(|_| CliError::ConfigError("Invalid user ID".to_string()))?;

    // Get or generate device ID from config
    let device_id = config
        .device
        .as_ref()
        .and_then(|d| uuid::Uuid::parse_str(&d.id).ok())
        .ok_or_else(|| CliError::ConfigError("Device ID not configured".to_string()))?;

    // Generate keypair for this device (in production, this should be stored)
    let keypair = Keypair::generate_ed25519();

    // Get network listen address from config
    let listen_addr = format!(
        "/ip4/{}/tcp/{}",
        config.network.listen_address,
        config.network.listen_port
    );

    let mut authorizer = AuthorizerWorkflow::new();

    let qr_data = authorizer
        .generate_qr_code(user_id, device_id, &keypair, listen_addr)
        .map_err(|e| CliError::AuthError(format!("Failed to generate QR code: {}", e)))?;

    output::success("QR Code generated successfully!");
    output::info(&format!("Valid for: 5 minutes"));
    output::info(&format!("Device type: {}", device_type));

    // Display QR code using qr2term if available
    #[cfg(feature = "cli")]
    {
        use qr2term::print_qr;
        output::info("\nScan this QR code with the new device:\n");
        if let Err(e) = print_qr(&qr_data) {
            output::warning(&format!("Could not display QR code: {}", e));
            output::info("QR code data (copy to new device):");
            output::info(&qr_data);
        }
    }
    #[cfg(not(feature = "cli"))]
    {
        output::info("QR code data (copy to new device):");
        output::info(&qr_data);
    }

    Ok(())
}

pub async fn authorize(code: &str, config: &Config) -> CliResult<()> {
    output::step("Authorizing device with code");

    // Scan the QR code data
    let challenge = NewDeviceWorkflow::scan_qr_code(code)
        .map_err(|e| CliError::AuthError(format!("Failed to scan QR code: {}", e)))?;

    output::info(&format!("Challenge ID: {}", challenge.challenge_id));
    output::info(&format!("Authorizer: {}", challenge.authorizer_peer_id));

    // Prompt for device information
    output::info("This device will be added to the account");

    // Generate keypair for this new device
    let new_keypair = Keypair::generate_ed25519();

    // Create pairing request with detected or default device type
    let device_type = detect_device_type();
    let device_name = format!(
        "Device-{}",
        uuid::Uuid::new_v4().to_string()[..8].to_string()
    );

    let auth_response = NewDeviceWorkflow::create_pairing_request(
        &challenge,
        device_type.clone(),
        device_name.clone(),
        &new_keypair,
    )
    .map_err(|e| CliError::AuthError(format!("Failed to create pairing request: {}", e)))?;

    output::info(&format!("Pairing request created for: {}", device_name));
    output::info("Connect to authorizer to complete pairing");

    // In a full implementation, this would:
    // 1. Establish P2P connection to authorizer
    // 2. Send auth_response
    // 3. Receive authorization result
    // 4. Save device credentials locally

    output::warning("Note: Full P2P connection not yet implemented");
    output::info("Save this device information:");
    output::info(&format!(
        "  Device ID: {}",
        auth_response.requesting_device_id
    ));

    Ok(())
}

pub async fn remove(device_id: &str, config: &Config) -> CliResult<()> {
    output::step(&format!("Removing device: {}", device_id));

    let device_uuid = uuid::Uuid::parse_str(device_id)
        .map_err(|_| CliError::ValidationError("Invalid device ID format".to_string()))?;

    let db_path = config.db_path();
    let conn = initialize_database(&db_path).map_err(|e| CliError::DatabaseError(e.to_string()))?;

    // Remove the device
    let rows_affected =
        delete_device(&conn, device_uuid).map_err(|e| CliError::DatabaseError(e.to_string()))?;

    if rows_affected > 0 {
        output::success(&format!("Device {} removed successfully", device_id));
    } else {
        output::warning(&format!("Device {} not found", device_id));
    }

    Ok(())
}
