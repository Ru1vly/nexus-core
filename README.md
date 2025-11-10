# nexus-core

[![Tests](https://img.shields.io/badge/tests-59%20passing-brightgreen)](tests/)
[![Rust](https://img.shields.io/badge/rust-nightly%202024-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)
[![Crates.io](https://img.shields.io/crates/v/nexus-core)](https://crates.io/crates/nexus-core)

> Core Rust library for the FocusSuite ecosystem - providing database operations, business logic, and peer-to-peer synchronization capabilities. Now available as a CLI tool for developers!

## Overview

`nexus-core` is a cross-platform Rust library and CLI tool that powers FocusSuite's productivity applications. It provides:

- **SQLite database** with automatic schema migrations
- **User authentication** with Argon2 password hashing
- **Task management** (lists, tasks, planner blocks)
- **Habit tracking** with streak calculations
- **Pomodoro timers** with customizable presets
- **Website/app blocking** for focus sessions
- **P2P synchronization** using libp2p with CRDT-based conflict resolution
- **Cross-platform builds** for iOS, Android, macOS, Windows, Linux, WatchOS, and WearOS

## Quick Start

### Prerequisites

- **Rust Nightly** (2024 edition features required)
- SQLite 3.x

```bash
# Install Rust nightly
rustup install nightly
rustup default nightly
```

### Installation

#### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
nexus-core = { path = "../nexus-core" }
```

#### As a CLI Tool

```bash
# Build and install
cargo install --path . --features cli

# Or build from source
cargo build --release --features cli

# Verify installation
nexus-cli --version
```

### Basic Usage

#### Library Usage

```rust
use nexus_core::{initialize_database, register_user, login_user};

// Initialize database (auto-migrates to latest schema)
let conn = initialize_database("nexus.db")?;

// Register a new user
let user = register_user(
    &conn,
    "alice".to_string(),
    "alice@example.com".to_string(),
    "secure_password".to_string(),
)?;

// Login
let authenticated_user = login_user(&conn, "alice", "secure_password")?;

println!("Welcome, {}!", authenticated_user.user_name);
```

#### CLI Usage

```bash
# Initialize with user
nexus-cli init --user alice --email alice@example.com

# Start sync daemon
nexus-cli start --daemon

# Check status
nexus-cli status

# View logs
nexus-cli logs --follow

# For complete CLI documentation:
# See docs/CLI_USAGE.md
```

## Features

### Database & Migrations

Automatic schema versioning and migrations:

```rust
use nexus_core::{get_current_version, get_migration_history};

let version = get_current_version(&conn)?;
println!("Schema version: {}", version);

let history = get_migration_history(&conn)?;
for (version, applied_at, description) in history {
    println!("v{}: {} ({})", version, description, applied_at);
}
```

See [docs/DATABASE_MIGRATIONS.md](docs/DATABASE_MIGRATIONS.md) for complete migration guide.

### Task Management

```rust
use nexus_core::{create_new_task_list, add_task_to_list, mark_task_as_complete};

// Create a task list
let list = create_new_task_list(&conn, user_id, device_id, "Work".to_string())?;

// Add tasks
let task = add_task_to_list(&conn, user_id, device_id, list.list_id, "Review PR".to_string())?;

// Mark complete
mark_task_as_complete(&conn, user_id, device_id, task.task_id)?;
```

### Habit Tracking

```rust
use nexus_core::{create_habit, log_habit_completion, get_habit_streak};
use chrono::Utc;

// Create a habit
let habit = create_habit(
    &conn,
    user_id,
    device_id,
    "Morning Run".to_string(),
    Some("Run 5km every morning".to_string()),
    None,
    "daily".to_string(),
)?;

// Log completion
let today = Utc::now().naive_utc().date();
log_habit_completion(&conn, user_id, device_id, habit.habit_id, today, None)?;

// Get streak
let streak = get_habit_streak(&conn, user_id, habit.habit_id)?;
println!("Current streak: {} days", streak);
```

### P2P Synchronization

```rust
use nexus_core::{create_swarm, sync_with_peer};

// Create libp2p swarm
let mut swarm = create_swarm().await?;

// Sync with peer
sync_with_peer(&mut swarm, peer_id, &conn, user_id, device_id).await?;
```

See [docs/P2P_SYNC.md](docs/P2P_SYNC.md) for detailed synchronization architecture (coming soon).

## Building

### Development Build

```bash
cargo build
cargo test
```

### Release Build

```bash
cargo build --release
```

### Cross-Compilation

Build for mobile and desktop platforms:

```bash
# Setup cross-compilation targets
make setup-targets

# Build for specific platforms
make build-ios
make build-android
make build-macos
make build-windows

# Build all supported platforms
make build-all
```

See [CROSS_COMPILATION.md](CROSS_COMPILATION.md) for detailed cross-compilation guide.

## Testing

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test --lib              # Unit tests
cargo test --test logic_test  # Logic tests
cargo test --test migration_test  # Migration tests

# Run with output
cargo test -- --nocapture
```

**Test Coverage:**
- 59 tests total
- 6 unit tests (library)
- 7 integration tests (database operations)
- 37 logic tests (business logic)
- 8 migration tests (schema versioning)
- 1 sync test (P2P synchronization)

## Documentation

| Document | Description |
|----------|-------------|
| [CLI_USAGE.md](docs/CLI_USAGE.md) | **Complete CLI tool guide** |
| [DATABASE_MIGRATIONS.md](docs/DATABASE_MIGRATIONS.md) | Complete migration system guide |
| [MIGRATION_QUICK_START.md](docs/MIGRATION_QUICK_START.md) | Quick reference for migrations |
| [MIGRATION_SYSTEM_SUMMARY.md](MIGRATION_SYSTEM_SUMMARY.md) | Implementation summary |
| [CROSS_COMPILATION.md](CROSS_COMPILATION.md) | Platform build instructions |
| [IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md) | Core API implementation details |

### API Documentation

Generate local API documentation:

```bash
cargo doc --open
```

## Architecture

```
nexus-core/
├── src/
│   ├── db/                 # Database layer
│   │   ├── operations.rs   # Low-level CRUD operations
│   │   ├── migrations.rs   # Schema migration system
│   │   └── migrations/     # SQL migration files
│   ├── logic/              # Business logic layer
│   │   ├── mod.rs          # User, task, habit logic
│   │   └── sync.rs         # P2P synchronization
│   ├── models.rs           # Data structures
│   ├── error.rs            # Error types
│   └── lib.rs              # Public API surface
├── tests/                  # Integration tests
├── scripts/                # Build scripts
└── docs/                   # Documentation
```

### Layer Responsibilities

1. **Database Layer** (`src/db/`): SQLite operations, migrations, schema management
2. **Business Logic** (`src/logic/`): Authentication, authorization, CRDT operations
3. **Models** (`src/models.rs`): Shared data structures
4. **Public API** (`src/lib.rs`): Re-exports and public interface

## Security

- **Password Hashing**: Argon2 with cryptographic salts
- **SQL Injection Prevention**: Parameterized queries only
- **Input Validation**: Comprehensive validation on all inputs
- **Access Control**: User ownership verification on all operations
- **Constant-Time Comparison**: Password verification uses timing-safe comparison

## Requirements

### Rust Version

**Requires Rust Nightly with 2024 edition features:**

The codebase uses Rust 2024 edition features (let chains), requiring nightly Rust:

```bash
rustup install nightly
rustup default nightly
```

### System Dependencies

- SQLite 3.x (bundled via rusqlite)
- OpenSSL (for libp2p on some platforms)

### Cross-Compilation Dependencies

See [CROSS_COMPILATION.md](CROSS_COMPILATION.md) for platform-specific requirements.

## Supported Platforms

| Platform | Architectures | Status |
|----------|--------------|--------|
| **iOS** | arm64, arm64-sim, x86_64-sim | ✅ Supported |
| **Android** | arm64-v8a, armeabi-v7a, x86, x86_64 | ✅ Supported |
| **macOS** | arm64 (M1/M2/M3), x86_64 (Intel) | ✅ Supported |
| **Windows** | x64, arm64 | ✅ Supported |
| **Linux** | x86_64, arm64 | ✅ Supported |
| **WatchOS** | arm64, arm64-sim | ✅ Supported |
| **WearOS** | arm64-v8a, armeabi-v7a | ✅ Supported |

## Performance

**Optimized for Size:**
- Binary size optimization enabled
- Link-time optimization (LTO)
- Debug symbols stripped
- Panic handler optimized

**Benchmarks** (coming soon)

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Format code: `cargo fmt`
6. Check for issues: `cargo clippy`
7. Submit a pull request

### Code Style

- Follow Rust standard naming conventions
- Document public APIs with doc comments
- Write comprehensive tests
- Handle all errors explicitly (no `unwrap()` in production code)

## Versioning

This project uses semantic versioning:

- **0.1.0**: Initial development release
- Database schema version tracked independently (see migrations)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Support

For issues, questions, or contributions:

- **Issues**: [GitHub Issues](https://github.com/kodfikirsanat/focussuite/issues)
- **Documentation**: [docs/](docs/)
- **Migration Guide**: [docs/DATABASE_MIGRATIONS.md](docs/DATABASE_MIGRATIONS.md)

## Acknowledgments

Built with:
- [Rust](https://www.rust-lang.org/) - Systems programming language
- [SQLite](https://www.sqlite.org/) - Embedded database
- [rusqlite](https://github.com/rusqlite/rusqlite) - Rust SQLite bindings
- [libp2p](https://libp2p.io/) - P2P networking
- [Argon2](https://github.com/P-H-C/phc-winner-argon2) - Password hashing
- [Chrono](https://github.com/chronotope/chrono) - Date and time
- [UUID](https://github.com/uuid-rs/uuid) - Unique identifiers
- [Serde](https://serde.rs/) - Serialization

---

**Part of the [FocusSuite](https://github.com/kodfikirsanat/focussuite) ecosystem**
