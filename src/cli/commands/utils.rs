use crate::cli::config::Config;
use crate::cli::errors::{CliError, CliResult};
use crate::cli::output;
use crate::db::operations::initialize_database;
use rusqlite::params;
use std::fs;

pub async fn query(sql: &str, json: bool, config: &Config) -> CliResult<()> {
    let db_path = config.db_path();
    let conn = initialize_database(&db_path)
        .map_err(|e| CliError::DatabaseError(e.to_string()))?;

    // Execute query
    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| CliError::DatabaseError(e.to_string()))?;

    let column_count = stmt.column_count();
    let column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

    let rows = stmt
        .query_map(params![], |row| {
            let mut values = Vec::new();
            for i in 0..column_count {
                let value: String = row
                    .get::<_, Option<String>>(i)
                    .unwrap_or(None)
                    .unwrap_or_else(|| "NULL".to_string());
                values.push(value);
            }
            Ok(values)
        })
        .map_err(|e| CliError::DatabaseError(e.to_string()))?;

    let mut results: Vec<Vec<String>> = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| CliError::DatabaseError(e.to_string()))?);
    }

    if json {
        let json_results: Vec<_> = results
            .iter()
            .map(|row| {
                let mut obj = serde_json::Map::new();
                for (i, col_name) in column_names.iter().enumerate() {
                    obj.insert(col_name.clone(), serde_json::Value::String(row[i].clone()));
                }
                serde_json::Value::Object(obj)
            })
            .collect();
        output::json(&serde_json::json!(json_results));
    } else {
        if results.is_empty() {
            output::info("No results");
            return Ok(());
        }

        let header_refs: Vec<&str> = column_names.iter().map(|s| s.as_str()).collect();
        let mut table = output::create_table(header_refs);

        for row in results {
            let cells: Vec<_> = row.iter().map(|v| prettytable::Cell::new(v)).collect();
            table.add_row(prettytable::Row::new(cells));
        }

        table.printstd();
    }

    Ok(())
}

pub async fn oplog(
    since: Option<i64>,
    device: Option<&str>,
    limit: usize,
    json: bool,
    config: &Config,
) -> CliResult<()> {
    let db_path = config.db_path();
    let conn = initialize_database(&db_path)
        .map_err(|e| CliError::DatabaseError(e.to_string()))?;

    let mut sql = "SELECT id, device_id, timestamp, table_name, op_type, data FROM oplog".to_string();
    let mut conditions = Vec::new();

    if let Some(ts) = since {
        conditions.push(format!("timestamp > {}", ts));
    }

    if let Some(dev) = device {
        conditions.push(format!("device_id = '{}'", dev));
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(&format!(" ORDER BY timestamp DESC LIMIT {}", limit));

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| CliError::DatabaseError(e.to_string()))?;

    let entries = stmt
        .query_map(params![], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })
        .map_err(|e| CliError::DatabaseError(e.to_string()))?;

    let mut results = Vec::new();
    for entry in entries {
        results.push(entry.map_err(|e| CliError::DatabaseError(e.to_string()))?);
    }

    if json {
        let json_results: Vec<_> = results
            .iter()
            .map(|(id, device_id, timestamp, table, op_type, data)| {
                serde_json::json!({
                    "id": id,
                    "device_id": device_id,
                    "timestamp": timestamp,
                    "table": table,
                    "op_type": op_type,
                    "data": data,
                })
            })
            .collect();
        output::json(&serde_json::json!(json_results));
    } else {
        if results.is_empty() {
            output::info("No oplog entries");
            return Ok(());
        }

        let mut table = output::create_table(vec!["Timestamp", "Table", "Op Type", "Device ID"]);

        for (_, device_id, timestamp, table_name, op_type, _) in results {
            table.add_row(prettytable::Row::new(vec![
                prettytable::Cell::new(&timestamp.to_string()),
                prettytable::Cell::new(&table_name),
                prettytable::Cell::new(&op_type),
                prettytable::Cell::new(&device_id[..8]), // Show first 8 chars
            ]));
        }

        table.printstd();
    }

    Ok(())
}

pub async fn info(json: bool) -> CliResult<()> {
    let version = env!("CARGO_PKG_VERSION");
    let system = sysinfo::System::new_all();

    if json {
        output::json(&serde_json::json!({
            "version": version,
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "hostname": hostname::get().ok().and_then(|h| h.into_string().ok()),
        }));
    } else {
        output::print_box(
            "Nexus CLI Information",
            vec![
                ("Version", version),
                ("OS", std::env::consts::OS),
                ("Architecture", std::env::consts::ARCH),
                ("Hostname", &hostname::get().ok().and_then(|h| h.into_string().ok()).unwrap_or_else(|| "Unknown".to_string())),
            ],
        );
    }

    Ok(())
}

pub async fn doctor(config: &Config) -> CliResult<()> {
    output::header("Running diagnostics");

    let mut issues = Vec::new();
    let mut checks_passed = 0;
    let total_checks = 5;

    // Check 1: Config file
    output::step("Checking configuration file");
    if Config::default_path().exists() {
        output::success("Configuration file exists");
        checks_passed += 1;
    } else {
        output::warning("Configuration file not found");
        issues.push("Run 'nexus-cli init' to create configuration");
    }

    // Check 2: Database
    output::step("Checking database");
    let db_path = config.db_path();
    if std::path::Path::new(&db_path).exists() {
        output::success("Database exists");
        checks_passed += 1;

        // Try to connect
        match initialize_database(&db_path) {
            Ok(_) => {
                output::success("Database connection successful");
                checks_passed += 1;
            }
            Err(e) => {
                output::error(&format!("Database connection failed: {}", e));
                issues.push("Database is corrupted or incompatible");
            }
        }
    } else {
        output::warning("Database not found");
        issues.push("Run 'nexus-cli init' to create database");
    }

    // Check 3: User configured
    output::step("Checking user configuration");
    if config.user.is_some() {
        output::success("User is configured");
        checks_passed += 1;
    } else {
        output::warning("User not configured");
        issues.push("Run 'nexus-cli init --user <NAME> --email <EMAIL>' to configure user");
    }

    // Check 4: Device configured
    output::step("Checking device configuration");
    if config.device.is_some() {
        output::success("Device is configured");
        checks_passed += 1;
    } else {
        output::warning("Device not configured");
        issues.push("Device should be configured during user initialization");
    }

    println!();
    output::header("Diagnostic Results");
    println!("Checks passed: {}/{}", checks_passed, total_checks);

    if !issues.is_empty() {
        println!();
        output::warning("Issues found:");
        for issue in issues {
            println!("  • {}", issue);
        }
    } else {
        println!();
        output::success("All checks passed! System is healthy.");
    }

    Ok(())
}

pub async fn export(path: &str, config: &Config) -> CliResult<()> {
    let db_path = config.db_path();

    output::step(&format!("Exporting database to {}", path));

    fs::copy(&db_path, path)?;

    output::success(&format!("Database exported to {}", path));

    Ok(())
}

pub async fn import(path: &str, force: bool, config: &Config) -> CliResult<()> {
    let db_path = config.db_path();

    if std::path::Path::new(&db_path).exists() && !force {
        return Err(CliError::ValidationError(
            "Database already exists. Use --force to overwrite".to_string(),
        ));
    }

    output::step(&format!("Importing database from {}", path));

    // Create parent directory if needed
    if let Some(parent) = std::path::Path::new(&db_path).parent() {
        fs::create_dir_all(parent)?;
    }

    fs::copy(path, &db_path)?;

    output::success(&format!("Database imported to {}", db_path));

    Ok(())
}
