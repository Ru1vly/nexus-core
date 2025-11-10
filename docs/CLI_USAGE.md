# Nexus CLI Usage Guide

The Nexus CLI is a command-line interface for managing and synchronizing data across devices using the Nexus synchronization engine.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Commands](#commands)
  - [Initialization](#initialization)
  - [Daemon Management](#daemon-management)
  - [Sync Operations](#sync-operations)
  - [Peer Management](#peer-management)
  - [Device Management](#device-management)
  - [Configuration](#configuration)
  - [Logs & Debugging](#logs--debugging)
  - [Utilities](#utilities)
- [Configuration File](#configuration-file)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/kodfikirsanat/focussuite
cd nexus-core

# Build with CLI feature
cargo build --release --features cli

# Install the binary
cargo install --path . --features cli

# Verify installation
nexus-cli --version
```

### From Cargo

```bash
cargo install nexus-cli --features cli
```

## Quick Start

1. **Initialize the database and create a user:**

```bash
nexus-cli init --user alice --email alice@example.com
```

You'll be prompted to enter a password.

2. **Start the sync daemon:**

```bash
# Foreground mode (recommended for first-time setup)
nexus-cli start

# Background mode
nexus-cli start --daemon
```

3. **Check sync status:**

```bash
nexus-cli status
```

4. **View logs:**

```bash
nexus-cli logs --follow
```

## Commands

### Initialization

#### `nexus-cli init`

Initialize a new Nexus database and configuration.

**Options:**
- `--path <PATH>` - Database path (default: `~/.nexus/nexus.db`)
- `--user <USERNAME>` - Username for initial user
- `--email <EMAIL>` - Email for initial user
- `--password <PASSWORD>` - Password (will prompt if not provided)

**Examples:**

```bash
# Initialize with user
nexus-cli init --user alice --email alice@example.com

# Initialize with custom database path
nexus-cli init --path /var/lib/nexus/db.sqlite --user alice --email alice@example.com

# Initialize database only (no user)
nexus-cli init
```

### Daemon Management

#### `nexus-cli start`

Start the synchronization daemon.

**Options:**
- `--daemon, -d` - Run in background mode
- `--port <PORT>` - Listen port (default: 0 for random)
- `--config <FILE>` - Configuration file path

**Examples:**

```bash
# Start in foreground (recommended for debugging)
nexus-cli start

# Start in background
nexus-cli start --daemon

# Start on specific port
nexus-cli start --port 9000
```

#### `nexus-cli stop`

Stop the running daemon.

```bash
nexus-cli stop
```

#### `nexus-cli restart`

Restart the daemon.

**Options:**
- `--daemon, -d` - Run in background after restart

```bash
# Restart in foreground
nexus-cli restart

# Restart in background
nexus-cli restart --daemon
```

#### `nexus-cli status`

Show daemon status and synchronization information.

**Options:**
- `--watch, -w` - Watch mode (continuous monitoring)
- `--interval <SECONDS>` - Update interval for watch mode (default: 2)
- `--json` - Output in JSON format

**Examples:**

```bash
# Single status check
nexus-cli status

# Watch mode (updates every 2 seconds)
nexus-cli status --watch

# Watch with custom interval
nexus-cli status --watch --interval 5

# JSON output
nexus-cli status --json
```

### Sync Operations

#### `nexus-cli sync`

Trigger a synchronization now.

**Options:**
- `--force, -f` - Force full synchronization

**Examples:**

```bash
# Trigger sync
nexus-cli sync

# Force full sync
nexus-cli sync --force
```

### Peer Management

#### `nexus-cli peer list`

List all connected peers.

**Options:**
- `--json` - Output in JSON format

```bash
# List peers
nexus-cli peer list

# JSON output
nexus-cli peer list --json
```

#### `nexus-cli peer add <MULTIADDR>`

Add a peer or bootstrap node.

```bash
nexus-cli peer add "/ip4/192.168.1.100/tcp/9000/p2p/12D3KooWABC..."
```

#### `nexus-cli peer remove <PEER_ID>`

Remove a peer.

```bash
nexus-cli peer remove "12D3KooWABC..."
```

#### `nexus-cli peer info <PEER_ID>`

Show detailed information about a peer.

**Options:**
- `--json` - Output in JSON format

```bash
# Show peer info
nexus-cli peer info "12D3KooWABC..."

# JSON output
nexus-cli peer info "12D3KooWABC..." --json
```

### Device Management

#### `nexus-cli device list`

List all user devices.

**Options:**
- `--json` - Output in JSON format

```bash
# List devices
nexus-cli device list

# JSON output
nexus-cli device list --json
```

#### `nexus-cli device pair`

Generate a QR code for pairing a new device.

**Options:**
- `--device-type <TYPE>` - Device type (default: mobile)
- `--name <NAME>` - Device name

```bash
# Generate pairing QR for mobile
nexus-cli device pair

# Generate for specific device type
nexus-cli device pair --device-type tablet --name "iPad Pro"
```

#### `nexus-cli device authorize <CODE>`

Authorize a device using a pairing code.

```bash
nexus-cli device authorize "ABC123DEF456"
```

#### `nexus-cli device remove <DEVICE_ID>`

Remove a device from the user account.

```bash
nexus-cli device remove "550e8400-e29b-41d4-a716-446655440000"
```

### Configuration

#### `nexus-cli config set <KEY> <VALUE>`

Set a configuration value.

**Examples:**

```bash
# Enable mDNS discovery
nexus-cli config set sync.enable_mdns true

# Set listen port
nexus-cli config set network.listen_port 9000

# Set log level
nexus-cli config set logging.level debug
```

#### `nexus-cli config get <KEY>`

Get a configuration value.

```bash
# Get database path
nexus-cli config get database.path

# Get sync status
nexus-cli config get sync.enabled
```

#### `nexus-cli config list`

List all configuration settings.

**Options:**
- `--json` - Output in JSON format

```bash
# List all config
nexus-cli config list

# JSON output
nexus-cli config list --json
```

#### `nexus-cli config edit`

Open the configuration file in your default editor.

```bash
nexus-cli config edit
```

The editor is determined by the `$EDITOR` environment variable (defaults to `vi` on Unix, `notepad` on Windows).

### Logs & Debugging

#### `nexus-cli logs`

View synchronization logs.

**Options:**
- `--follow, -f` - Follow log output (like `tail -f`)
- `--lines, -n <NUM>` - Number of lines to show (default: 50)
- `--level <LEVEL>` - Filter by log level (trace, debug, info, warn, error)

**Examples:**

```bash
# View last 50 lines
nexus-cli logs

# Follow logs in real-time
nexus-cli logs --follow

# Show last 100 lines
nexus-cli logs --lines 100

# Filter by level
nexus-cli logs --level error --follow
```

#### `nexus-cli query <SQL>`

Execute a SQL query on the database (for debugging).

**Options:**
- `--json` - Output in JSON format

**Examples:**

```bash
# Query users
nexus-cli query "SELECT * FROM users"

# Count tasks
nexus-cli query "SELECT COUNT(*) FROM tasks"

# JSON output
nexus-cli query "SELECT * FROM users" --json
```

**⚠️ Warning:** This command provides direct database access. Use with caution!

#### `nexus-cli oplog`

View the operation log (sync history).

**Options:**
- `--since <TIMESTAMP>` - Show entries since timestamp
- `--device <DEVICE_ID>` - Filter by device ID
- `--limit <NUM>` - Number of entries to show (default: 50)
- `--json` - Output in JSON format

**Examples:**

```bash
# View recent operations
nexus-cli oplog

# View last 100 operations
nexus-cli oplog --limit 100

# View operations from specific device
nexus-cli oplog --device "550e8400-e29b-41d4-a716-446655440000"

# View operations since timestamp
nexus-cli oplog --since 1704067200

# JSON output
nexus-cli oplog --json
```

### Utilities

#### `nexus-cli info`

Show system and version information.

**Options:**
- `--json` - Output in JSON format

```bash
# Show info
nexus-cli info

# JSON output
nexus-cli info --json
```

#### `nexus-cli doctor`

Run diagnostic checks on the system.

```bash
nexus-cli doctor
```

This command checks:
- Configuration file existence
- Database existence and connectivity
- User configuration
- Device configuration

#### `nexus-cli export <PATH>`

Export the database to a file.

```bash
nexus-cli export backup.db
```

#### `nexus-cli import <PATH>`

Import a database from a file.

**Options:**
- `--force, -f` - Overwrite existing database

```bash
# Import database
nexus-cli import backup.db

# Force overwrite
nexus-cli import backup.db --force
```

## Configuration File

The configuration file is located at `~/.nexus/config.toml` by default.

### Configuration Structure

```toml
[database]
path = "~/.nexus/nexus.db"
auto_migrate = true

[user]
id = "550e8400-e29b-41d4-a716-446655440000"
name = "alice"
email = "alice@example.com"

[device]
id = "660e8400-e29b-41d4-a716-446655440001"
type = "cli"
name = "my-laptop"

[sync]
enabled = true
auto_start = false
enable_mdns = true
enable_relay = true
heartbeat_interval_secs = 10
max_message_size = 65536

[network]
listen_port = 0  # 0 = random port
listen_address = "0.0.0.0"
bootstrap_nodes = []
relay_servers = []

[logging]
level = "info"  # trace, debug, info, warn, error
format = "pretty"  # pretty, json, compact
file = "~/.nexus/nexus.log"
max_size_mb = 100
max_files = 5
```

### Configuration Keys

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `database.path` | string | `~/.nexus/nexus.db` | Database file path |
| `database.auto_migrate` | boolean | `true` | Auto-apply database migrations |
| `sync.enabled` | boolean | `true` | Enable synchronization |
| `sync.auto_start` | boolean | `false` | Auto-start daemon on login |
| `sync.enable_mdns` | boolean | `true` | Enable mDNS peer discovery |
| `sync.enable_relay` | boolean | `true` | Enable relay servers |
| `sync.heartbeat_interval_secs` | integer | `10` | Heartbeat interval in seconds |
| `sync.max_message_size` | integer | `65536` | Max sync message size in bytes |
| `network.listen_port` | integer | `0` | Listen port (0 = random) |
| `network.listen_address` | string | `"0.0.0.0"` | Listen address |
| `network.bootstrap_nodes` | array | `[]` | Bootstrap node multiaddresses |
| `network.relay_servers` | array | `[]` | Relay server multiaddresses |
| `logging.level` | string | `"info"` | Log level |
| `logging.format` | string | `"pretty"` | Log format |
| `logging.file` | string | `~/.nexus/nexus.log` | Log file path |
| `logging.max_size_mb` | integer | `100` | Max log file size in MB |
| `logging.max_files` | integer | `5` | Max number of log files |

## Examples

### Complete Setup Workflow

```bash
# 1. Initialize with user
nexus-cli init --user alice --email alice@example.com

# 2. Configure bootstrap nodes (optional)
nexus-cli config set network.bootstrap_nodes '["node1.example.com", "node2.example.com"]'

# 3. Start daemon in foreground (for testing)
nexus-cli start

# In another terminal:
# 4. Check status
nexus-cli status

# 5. View logs
nexus-cli logs --follow

# 6. Once verified, stop foreground daemon
# (Ctrl+C in the daemon terminal)

# 7. Start in background
nexus-cli start --daemon

# 8. Verify it's running
nexus-cli status
```

### Monitoring and Debugging

```bash
# Watch status continuously
nexus-cli status --watch

# Follow logs with error filter
nexus-cli logs --follow --level error

# Check system health
nexus-cli doctor

# View recent sync operations
nexus-cli oplog --limit 20

# Query database
nexus-cli query "SELECT COUNT(*) as total FROM tasks"
```

### Multi-Device Setup

On the primary device:
```bash
# Generate pairing QR code
nexus-cli device pair
```

On the new device:
```bash
# Initialize and authorize
nexus-cli init
nexus-cli device authorize "PAIRING_CODE_FROM_QR"
nexus-cli start --daemon
```

### Backup and Restore

```bash
# Backup database
nexus-cli export ~/backups/nexus-backup-$(date +%Y%m%d).db

# Restore database
nexus-cli stop
nexus-cli import ~/backups/nexus-backup-20250109.db --force
nexus-cli start --daemon
```

## Troubleshooting

### Daemon Won't Start

1. Check if already running:
   ```bash
   nexus-cli status
   ```

2. Check configuration:
   ```bash
   nexus-cli config list
   nexus-cli doctor
   ```

3. Check logs:
   ```bash
   nexus-cli logs --lines 100
   ```

4. Try foreground mode for debugging:
   ```bash
   nexus-cli start
   ```

### Sync Not Working

1. Check daemon status:
   ```bash
   nexus-cli status
   ```

2. Check peer connections:
   ```bash
   nexus-cli peer list
   ```

3. Check logs for errors:
   ```bash
   nexus-cli logs --follow --level error
   ```

4. Verify network configuration:
   ```bash
   nexus-cli config get network.listen_port
   nexus-cli config get sync.enable_mdns
   ```

### Database Corruption

1. Stop daemon:
   ```bash
   nexus-cli stop
   ```

2. Export current database (if possible):
   ```bash
   nexus-cli export ~/nexus-damaged.db
   ```

3. Restore from backup:
   ```bash
   nexus-cli import ~/backups/nexus-backup-latest.db --force
   ```

4. Restart daemon:
   ```bash
   nexus-cli start --daemon
   ```

### Configuration Issues

1. Check configuration file:
   ```bash
   cat ~/.nexus/config.toml
   ```

2. Reset to defaults:
   ```bash
   rm ~/.nexus/config.toml
   nexus-cli init
   ```

3. Edit configuration:
   ```bash
   nexus-cli config edit
   ```

## Global Options

All commands support these global options:

- `--config, -c <FILE>` - Use custom configuration file
- `--verbose, -v` - Enable verbose output
- `--json` - Output in JSON format (where supported)
- `--help, -h` - Show help for command
- `--version, -V` - Show version information

**Examples:**

```bash
# Use custom config file
nexus-cli --config /etc/nexus/config.toml status

# Verbose output
nexus-cli --verbose start

# Show version
nexus-cli --version
```

## Environment Variables

- `RUST_LOG` - Control log level (set automatically by `--verbose`)
- `EDITOR` - Editor to use for `nexus-cli config edit`

**Examples:**

```bash
# Set custom editor
export EDITOR=nano
nexus-cli config edit

# Override log level
RUST_LOG=debug nexus-cli start
```

## Exit Codes

- `0` - Success
- `1` - Error

## Getting Help

For command-specific help:

```bash
nexus-cli <command> --help
```

For general help:

```bash
nexus-cli --help
```

For more information, see:
- [Main README](../README.md)
- [Database Migrations](./DATABASE_MIGRATIONS.md)
- [Installation Guide](../INSTALL.md)
