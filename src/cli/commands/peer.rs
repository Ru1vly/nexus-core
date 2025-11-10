use crate::cli::config::Config;
use crate::cli::errors::{CliError, CliResult};
use crate::cli::output;
use crate::db::operations::{get_all_peers, initialize_database};

pub async fn list(json: bool, config: &Config) -> CliResult<()> {
    let db_path = config.db_path();
    let conn = initialize_database(&db_path)
        .map_err(|e| CliError::DatabaseError(e.to_string()))?;

    let peers = get_all_peers(&conn)
        .map_err(|e| CliError::DatabaseError(e.to_string()))?;

    if json {
        let peers_json: Vec<_> = peers
            .iter()
            .map(|p| {
                serde_json::json!({
                    "peer_id": p.peer_id.to_string(),
                    "user_id": p.user_id.to_string(),
                    "device_id": p.device_id.to_string(),
                    "last_known_ip": p.last_known_ip,
                    "last_sync_time": p.last_sync_time,
                })
            })
            .collect();
        output::json(&serde_json::json!(peers_json));
    } else {
        if peers.is_empty() {
            output::info("No peers connected");
            return Ok(());
        }

        let mut table = output::create_table(vec!["Peer ID", "Device ID", "Last IP", "Last Sync"]);

        for peer in peers {
            table.add_row(prettytable::Row::new(vec![
                prettytable::Cell::new(&peer.peer_id.to_string()),
                prettytable::Cell::new(&peer.device_id.to_string()),
                prettytable::Cell::new(peer.last_known_ip.as_deref().unwrap_or("N/A")),
                prettytable::Cell::new(&peer.last_sync_time.map(|t| t.to_string()).unwrap_or_else(|| "Never".to_string())),
            ]));
        }

        table.printstd();
    }

    Ok(())
}

pub async fn add(multiaddr: &str, config: &Config) -> CliResult<()> {
    output::step(&format!("Adding peer: {}", multiaddr));

    // TODO: Implement peer addition via IPC to daemon
    output::warning("Peer addition not yet implemented");
    output::info("Add bootstrap nodes to config: nexus-cli config set network.bootstrap_nodes");

    Ok(())
}

pub async fn remove(peer_id: &str, config: &Config) -> CliResult<()> {
    output::step(&format!("Removing peer: {}", peer_id));

    // TODO: Implement peer removal
    output::warning("Peer removal not yet implemented");

    Ok(())
}

pub async fn info(peer_id: &str, json: bool, config: &Config) -> CliResult<()> {
    let db_path = config.db_path();
    let conn = initialize_database(&db_path)
        .map_err(|e| CliError::DatabaseError(e.to_string()))?;

    let peer_uuid = uuid::Uuid::parse_str(peer_id)
        .map_err(|_| CliError::ValidationError("Invalid peer ID".to_string()))?;

    let peer = crate::db::operations::get_peer(&conn, peer_uuid)
        .map_err(|e| CliError::DatabaseError(e.to_string()))?;

    if json {
        output::json(&serde_json::json!({
            "peer_id": peer.peer_id.to_string(),
            "user_id": peer.user_id.to_string(),
            "device_id": peer.device_id.to_string(),
            "last_known_ip": peer.last_known_ip,
            "last_sync_time": peer.last_sync_time,
        }));
    } else {
        output::print_box(
            "Peer Information",
            vec![
                ("Peer ID", &peer.peer_id.to_string()),
                ("User ID", &peer.user_id.to_string()),
                ("Device ID", &peer.device_id.to_string()),
                ("Last Known IP", peer.last_known_ip.as_deref().unwrap_or("N/A")),
                ("Last Sync", &peer.last_sync_time.map(|t| t.to_string()).unwrap_or_else(|| "Never".to_string())),
            ],
        );
    }

    Ok(())
}
