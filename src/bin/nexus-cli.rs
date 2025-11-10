use clap::{Parser, Subcommand};
use nexus_core::cli::{commands, config::Config, errors::CliResult, output};
use std::process;

#[derive(Parser)]
#[command(name = "nexus-cli")]
#[command(version, about = "CLI tool for Nexus synchronization engine", long_about = None)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Output in JSON format
    #[arg(long)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Nexus database
    Init {
        /// Database path
        #[arg(short, long, value_name = "PATH")]
        path: Option<String>,

        /// Username for initial user
        #[arg(short, long)]
        user: Option<String>,

        /// Email for initial user
        #[arg(short, long)]
        email: Option<String>,

        /// Password for initial user
        #[arg(short = 'P', long)]
        password: Option<String>,
    },

    /// Start the sync daemon
    Start {
        /// Run in background as daemon
        #[arg(short, long)]
        daemon: bool,

        /// Port to listen on (0 = random)
        #[arg(short, long, default_value = "0")]
        port: u16,

        /// Configuration file
        #[arg(short, long)]
        config: Option<String>,
    },

    /// Stop the sync daemon
    Stop,

    /// Restart the sync daemon
    Restart {
        /// Run in background as daemon
        #[arg(short, long)]
        daemon: bool,
    },

    /// Show sync status
    Status {
        /// Watch mode (continuous monitoring)
        #[arg(short, long)]
        watch: bool,

        /// Update interval in seconds for watch mode
        #[arg(short, long, default_value = "2")]
        interval: u64,
    },

    /// Trigger a sync now
    Sync {
        /// Force full sync
        #[arg(short, long)]
        force: bool,
    },

    /// Peer management commands
    #[command(subcommand)]
    Peer(PeerCommands),

    /// Device management commands
    #[command(subcommand)]
    Device(DeviceCommands),

    /// Configuration management
    #[command(subcommand)]
    Config(ConfigCommands),

    /// View logs
    Logs {
        /// Follow log output
        #[arg(short, long)]
        follow: bool,

        /// Number of lines to show
        #[arg(short, long, default_value = "50")]
        lines: usize,

        /// Filter by log level
        #[arg(short = 'L', long)]
        level: Option<String>,
    },

    /// Execute SQL query (debugging)
    Query {
        /// SQL query to execute
        sql: String,
    },

    /// View operation log
    Oplog {
        /// Show entries since timestamp
        #[arg(long)]
        since: Option<i64>,

        /// Filter by device ID
        #[arg(long)]
        device: Option<String>,

        /// Number of entries to show
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },

    /// Show system information
    Info,

    /// Diagnose system issues
    Doctor,

    /// Export database
    Export {
        /// Output path
        path: String,
    },

    /// Import database
    Import {
        /// Input path
        path: String,

        /// Force overwrite existing database
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Subcommand)]
enum PeerCommands {
    /// List connected peers
    List,

    /// Add a peer or bootstrap node
    Add {
        /// Multiaddress of the peer
        multiaddr: String,
    },

    /// Remove a peer
    Remove {
        /// Peer ID to remove
        peer_id: String,
    },

    /// Show peer information
    Info {
        /// Peer ID to query
        peer_id: String,
    },
}

#[derive(Subcommand)]
enum DeviceCommands {
    /// List user devices
    List,

    /// Generate device pairing QR code
    Pair {
        /// Device type
        #[arg(short, long, default_value = "mobile")]
        device_type: String,

        /// Device name
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Authorize a device with pairing code
    Authorize {
        /// Authorization code
        code: String,
    },

    /// Remove a device
    Remove {
        /// Device ID to remove
        device_id: String,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,

        /// Configuration value
        value: String,
    },

    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },

    /// List all configuration
    List,

    /// Edit configuration file
    Edit,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize logging
    unsafe {
        if cli.verbose {
            std::env::set_var("RUST_LOG", "debug");
        } else {
            std::env::set_var("RUST_LOG", "info");
        }
    }
    env_logger::init();

    // Load configuration
    let config = match Config::load(cli.config.as_deref()) {
        Ok(cfg) => cfg,
        Err(e) => {
            // If config load fails, use default for init command
            match cli.command {
                Commands::Init { .. } => Config::default(),
                _ => {
                    output::error(&format!("Failed to load configuration: {}", e));
                    process::exit(1);
                }
            }
        }
    };

    // Execute command
    let result = match cli.command {
        Commands::Init {
            path,
            user,
            email,
            password,
        } => {
            commands::init::execute(path.as_deref(), user.as_deref(), email.as_deref(), password.as_deref()).await
        }
        Commands::Start { daemon, port, config: config_path } => {
            commands::daemon::start(daemon, port, config_path.as_deref(), &config).await
        }
        Commands::Stop => commands::daemon::stop(&config).await,
        Commands::Restart { daemon } => commands::daemon::restart(daemon, &config).await,
        Commands::Status { watch, interval } => {
            commands::daemon::status(watch, interval, cli.json, &config).await
        }
        Commands::Sync { force } => commands::sync::sync(force, &config).await,
        Commands::Peer(peer_cmd) => match peer_cmd {
            PeerCommands::List => commands::peer::list(cli.json, &config).await,
            PeerCommands::Add { multiaddr } => commands::peer::add(&multiaddr, &config).await,
            PeerCommands::Remove { peer_id } => commands::peer::remove(&peer_id, &config).await,
            PeerCommands::Info { peer_id } => commands::peer::info(&peer_id, cli.json, &config).await,
        },
        Commands::Device(device_cmd) => match device_cmd {
            DeviceCommands::List => commands::device::list(cli.json, &config).await,
            DeviceCommands::Pair { device_type, name } => {
                commands::device::pair(&device_type, name.as_deref(), &config).await
            }
            DeviceCommands::Authorize { code } => commands::device::authorize(&code, &config).await,
            DeviceCommands::Remove { device_id } => commands::device::remove(&device_id, &config).await,
        },
        Commands::Config(config_cmd) => match config_cmd {
            ConfigCommands::Set { key, value } => commands::config::set(&key, &value, &config).await,
            ConfigCommands::Get { key } => commands::config::get(&key, &config).await,
            ConfigCommands::List => commands::config::list(cli.json, &config).await,
            ConfigCommands::Edit => commands::config::edit(&config).await,
        },
        Commands::Logs {
            follow,
            lines,
            level,
        } => commands::logs::view(follow, lines, level.as_deref(), &config).await,
        Commands::Query { sql } => commands::utils::query(&sql, cli.json, &config).await,
        Commands::Oplog {
            since,
            device,
            limit,
        } => commands::utils::oplog(since, device.as_deref(), limit, cli.json, &config).await,
        Commands::Info => commands::utils::info(cli.json).await,
        Commands::Doctor => commands::utils::doctor(&config).await,
        Commands::Export { path } => commands::utils::export(&path, &config).await,
        Commands::Import { path, force } => commands::utils::import(&path, force, &config).await,
    };

    // Handle result
    match result {
        Ok(_) => {}
        Err(e) => {
            output::error(&format!("{}", e));
            process::exit(1);
        }
    }
}
