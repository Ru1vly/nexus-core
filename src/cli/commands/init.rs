use crate::cli::config::{Config, DeviceConfig, UserConfig};
use crate::cli::errors::{CliError, CliResult};
use crate::cli::output;
use crate::db::operations::initialize_database;
use crate::logic;
use std::fs;
use uuid::Uuid;

pub async fn execute(
    db_path: Option<&str>,
    username: Option<&str>,
    email: Option<&str>,
    password: Option<&str>,
) -> CliResult<()> {
    output::header("Initializing Nexus");

    // Load or create default config
    let mut config = Config::default();

    // Set database path if provided
    if let Some(path) = db_path {
        config.database.path = path.to_string();
    }

    // Expand the database path
    let db_path_expanded = Config::expand_path(&config.database.path);

    // Create parent directory if needed
    if let Some(parent) = std::path::Path::new(&db_path_expanded).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| CliError::IoError(e))?;
    }

    // Initialize database
    output::step(&format!("Creating database at {}", db_path_expanded));
    let conn = initialize_database(&db_path_expanded)
        .map_err(|e| CliError::DatabaseError(e.to_string()))?;
    output::success("Database initialized");

    // Create user if credentials provided
    if let (Some(user), Some(mail)) = (username, email) {
        output::step(&format!("Creating user '{}'", user));

        // Get password (prompt if not provided)
        let pass = if let Some(p) = password {
            p.to_string()
        } else {
            rpassword::prompt_password("Password: ")
                .map_err(|e| CliError::IoError(e))?
        };

        let user_obj = logic::register_user(&conn, user.to_string(), mail.to_string(), pass)
            .map_err(|e| CliError::DatabaseError(e))?;

        output::success(&format!("User '{}' created ({})", user, user_obj.user_id));

        // Generate device ID and register
        output::step("Registering CLI device");
        let device_name = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "cli-device".to_string());

        let device = logic::add_device_to_user(
            &conn,
            user_obj.user_id,
            "cli".to_string(),
            None,
        )
        .map_err(|e| CliError::DatabaseError(e))?;

        output::success(&format!("Device registered ({})", device.device_id));

        // Update config with user and device info
        config.user = Some(UserConfig {
            id: user_obj.user_id.to_string(),
            name: user_obj.user_name.clone(),
            email: user_obj.user_mail.clone(),
        });

        config.device = Some(DeviceConfig {
            id: device.device_id.to_string(),
            device_type: "cli".to_string(),
            name: device_name,
        });
    }

    // Save configuration
    output::step("Saving configuration");
    config.save(None)?;
    output::success(&format!(
        "Configuration saved to {}",
        Config::default_path().display()
    ));

    // Print summary
    println!();
    output::header("Setup Complete");
    output::key_value("Database", &db_path_expanded);
    output::key_value("Config", &Config::default_path().display().to_string());

    if config.user.is_some() {
        output::key_value("User", username.unwrap_or("N/A"));
        output::key_value("Device", &config.device.as_ref().map(|d| d.id.as_str()).unwrap_or("N/A"));
    } else {
        println!();
        output::info("No user created. Run 'nexus-cli init --user <USERNAME> --email <EMAIL>' to create a user.");
    }

    println!();
    output::info("Next steps:");
    println!("  1. Start sync daemon: nexus-cli start --daemon");
    println!("  2. Check status: nexus-cli status");
    println!("  3. View logs: nexus-cli logs --follow");

    Ok(())
}
