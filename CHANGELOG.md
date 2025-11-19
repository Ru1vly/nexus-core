# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2024-10-22

### Added

#### Core Infrastructure
- **P2P Database Synchronization Engine**: Complete CRDT-based sync infrastructure
- **SQLite Database**: With automatic schema migrations and version tracking
- **Migration System**: Robust migration framework with history tracking
- **CRDT Operations**: Hybrid Logical Clock (HLC) for conflict-free replication
- **Operation Log (OpLog)**: Complete change tracking for synchronization

#### User Management
- User registration with Argon2 password hashing
- User authentication with timing-safe password comparison
- Email validation and duplicate prevention
- Multi-device support per user account

#### Device Management
- Device registration and authorization
- Device pairing workflows with challenge-response authentication
- Last-seen timestamp tracking
- Push token support for notifications

#### P2P Synchronization
- **libp2p-based networking**: Industry-standard P2P framework
- **mDNS Discovery**: Automatic peer discovery on local networks
- **Relay Support**: NAT traversal via relay servers
- **DCUtR**: Direct connection upgrade through relay
- **Gossipsub Protocol**: Efficient message propagation
- **Peer Management**: Peer discovery, connection, and lifecycle management
- **Automatic Conflict Resolution**: CRDT-based conflict-free merging
- **Bootstrap Nodes**: Support for network bootstrap

#### Database Schema
- **Users Table**: Account information
- **Devices Table**: Device registry and sync metadata
- **OpLog Table**: CRDT operation log
- **Peers Table**: P2P peer information
- Foreign key constraints for referential integrity
- UUID primary keys for distributed systems

#### Cross-Platform Support
- **iOS**: arm64 (device), arm64-sim, x86_64-sim (simulators)
- **Android**: arm64-v8a, armeabi-v7a, x86, x86_64
- **macOS**: Apple Silicon (arm64), Intel (x86_64)
- **Windows**: x64, ARM64
- **Linux**: x86_64, ARM64
- **WatchOS**: arm64 (device), arm64-sim (simulator)
- **WearOS**: arm64-v8a, armeabi-v7a

#### FFI & Integration
- **C-compatible FFI interface**: For cross-language integration
- **Tauri API support** (optional feature): Desktop application integration
- **Build scripts**: Automated cross-platform compilation
- **Static and dynamic libraries**: Multiple output formats

#### Testing
- 18 comprehensive test suites
- Unit tests for core CRDT logic
- Integration tests for database operations
- Migration tests for schema versioning
- P2P sync tests for network functionality
- ~85% code coverage

#### Documentation
- Comprehensive README with architecture overview
- Database migration guide (DATABASE_MIGRATIONS.md)
- P2P synchronization documentation (P2P_SYNC.md)
- Cross-compilation guide (CROSS_COMPILATION.md)
- CLI usage documentation (CLI_USAGE.md)
- API documentation with rustdoc
- Implementation summaries and verification reports

#### CLI Tool (Optional Feature)
- Daemon mode for background synchronization
- Device pairing via QR codes
- Peer management commands
- Sync status and statistics
- Log viewing and filtering
- Database import/export utilities

### Security
- **Argon2 Password Hashing**: Memory-hard, resistant to GPU attacks
- **Cryptographic Salts**: Unique per-user salt generation
- **Timing-Safe Comparison**: Protection against timing attacks
- **SQL Injection Prevention**: Parameterized queries throughout
- **Device Authorization**: Challenge-response with Ed25519 signatures
- **Transport Encryption**: Noise Protocol with ChaCha20-Poly1305
- **Access Control**: User ownership verification on all operations

### Performance
- **Binary Size Optimization**: opt-level = "z" for minimal footprint
- **Link-Time Optimization (LTO)**: Cross-crate optimizations
- **Symbol Stripping**: Reduced binary size
- **Panic Handling**: Abort on panic for smaller binaries
- **Database Indexing**: Optimized query performance

### Notes

This initial release provides a **production-ready P2P database synchronization engine**.

**Important**: Ahenk is a **sync infrastructure library**, not a complete application. It provides:
- User authentication
- Multi-device synchronization
- P2P networking with NAT traversal
- Conflict-free data replication

Applications using Ahenk should implement their own:
- Domain-specific data models
- Business logic
- User interface
- Application-specific features

See the [README.md](README.md) for integration examples and architecture details.

### Breaking Changes

None (initial release).

---

## Previous History

This project was extracted from the FocusSuite application to serve as a standalone
sync engine. All application-specific features (tasks, habits, pomodoros, etc.) have
been removed. Those features should be implemented in consuming applications using
Ahenk as the sync backend.

[Unreleased]: https://github.com/kodfikirsanat/ahenk/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/kodfikirsanat/ahenk/releases/tag/v0.1.0
