# P2P Synchronization Architecture

Detailed technical documentation for Ahenk's peer-to-peer synchronization system.

## Overview

Ahenk uses a decentralized P2P architecture based on libp2p for synchronizing data across devices without requiring a central server.

## Architecture

### Network Stack

```
┌─────────────────────────────────────────────────┐
│          Application Layer                      │
│  (OpLog sync, device authorization)            │
├─────────────────────────────────────────────────┤
│          Protocol Layer                         │
│  - SyncProtocol (request/response)             │
│  - Heartbeat (keep-alive)                       │
│  - Discovery (announce presence)                │
├─────────────────────────────────────────────────┤
│          Transport Layer                        │
│  - mDNS (local discovery)                      │
│  - Gossipsub (message propagation)             │
│  - Relay (NAT traversal)                        │
│  - DCUtR (hole punching)                        │
├─────────────────────────────────────────────────┤
│          Security Layer                         │
│  - Noise Protocol (encryption)                  │
│  - Ed25519 (authentication)                     │
└─────────────────────────────────────────────────┘
```

### Components

#### 1. AhenkBehaviour

Network behavior that combines multiple libp2p protocols:

```rust
pub struct AhenkBehaviour {
    pub mdns: mdns::tokio::Behaviour,           // Local discovery
    pub gossipsub: gossipsub::Behaviour,        // Message propagation
    pub relay_client: relay::client::Behaviour, // NAT traversal
    pub dcutr: dcutr::Behaviour,               // Hole punching
}
```

#### 2. Peer Discovery

**Local Network (mDNS)**:
- Broadcasts presence on local network
- Discovers peers on same WiFi/LAN
- Zero configuration required
- Fastest connection method

**Global Network (Relay)**:
- Connects through relay servers
- Works across different networks
- Enables NAT traversal
- Fallback for direct connections

**DCUtR (Direct Connection Upgrade)**:
- Attempts to establish direct connection
- Uses relay for coordination
- Reduces relay server load
- Better performance and privacy

#### 3. Message Protocol

Sync messages use JSON-encoded structures:

```rust
pub enum SyncMessage {
    // Handshake
    Hello {
        peer_id: PeerId,
        device_id: Uuid,
        user_id: Uuid,
        schema_version: i32,
    },

    // Request operations
    RequestOps {
        since: i64,  // HLC timestamp
    },

    // Send operations
    SendOps {
        operations: Vec<OplogEntry>,
    },

    // Heartbeat
    Ping { timestamp: i64 },
    Pong { timestamp: i64 },

    // Error
    Error { message: String },
}
```

## Synchronization Flow

### Initial Sync

```
Device A                           Device B
   |                                  |
   |------ Hello (device_id, user) ->|
   |<----- Hello (device_id, user) --|
   |                                  |
   | [Verify both devices belong     |
   |  to same user account]          |
   |                                  |
   |------ RequestOps (since: 0) --->|
   |<----- SendOps (all ops) ---------|
   |                                  |
   |------ RequestOps (since: 0) --->|
   |<----- SendOps (all ops) ---------|
   |                                  |
   | [Both devices now have          |
   |  complete operation history]    |
```

### Incremental Sync

```
Device A                           Device B
   |                                  |
   | [Device A creates new operation] |
   |                                  |
   |------ SendOps (new ops) -------->|
   |                                  |
   | [Device B applies operations]   |
   | [Device B sends back ACK]       |
   |                                  |
   |<----- Pong -----------------------|
```

## CRDT Conflict Resolution

### Hybrid Logical Clock (HLC)

Provides causal ordering without clock synchronization:

```rust
pub struct HybridLogicalClock {
    physical: i64,  // System time (milliseconds)
    logical: u16,   // Counter for simultaneous events
}
```

**Properties**:
- Monotonically increasing
- Captures causality
- Tolerates clock skew
- Deterministic ordering

**Example**:
```
Device A (time 1000): op1 → HLC{1000, 0}
Device B (time 999):  op2 → HLC{999, 0}

After sync:
Both devices order: op2 (999,0) < op1 (1000,0)
```

### Last-Write-Wins (LWW)

Conflict resolution strategy:

```rust
fn resolve_conflict(local: OplogEntry, remote: OplogEntry) -> OplogEntry {
    if remote.timestamp > local.timestamp {
        remote  // Remote wins
    } else if remote.timestamp == local.timestamp {
        // Tie-breaker: higher device_id wins (deterministic)
        if remote.device_id > local.device_id {
            remote
        } else {
            local
        }
    } else {
        local  // Local wins
    }
}
```

### Operation Merging

```rust
pub fn merge(conn: &mut Connection, remote_ops: &[OplogEntry]) -> Result<()> {
    conn.execute("BEGIN TRANSACTION", [])?;

    for remote_op in remote_ops {
        // Check if operation already exists
        let exists = check_oplog_exists(conn, &remote_op.id)?;

        if !exists {
            // Insert new operation
            insert_oplog(conn, remote_op)?;

            // Apply to tables
            apply_oplog_entry(conn, remote_op)?;
        }
    }

    conn.execute("COMMIT", [])?;
    Ok(())
}
```

## NAT Traversal

### The Challenge

```
Home Network (NAT):
  Phone: 192.168.1.100 (private)
  Router: 203.0.113.45 (public)

Office Network (NAT):
  Laptop: 10.0.0.50 (private)
  Router: 198.51.100.23 (public)

Problem: Devices can't directly connect
```

### Solution Strategies

#### 1. Relay Server

```
Phone → Relay Server ← Laptop
       (203.0.113.100)

- Always works
- Higher latency
- Uses relay bandwidth
- Privacy concern (relay sees traffic)
```

#### 2. DCUtR (Hole Punching)

```
Step 1: Both connect to relay
Phone → Relay ← Laptop

Step 2: Relay coordinates

Step 3: Simultaneous connect
Phone ←────────→ Laptop
     (direct P2P)

- Lower latency
- Better privacy
- Saves relay bandwidth
- May fail with strict NAT
```

### Implementation

```rust
// Create swarm with relay support
let config = P2PConfig {
    enable_mdns: true,
    enable_relay: true,
    relay_servers: vec![
        "/ip4/relay1.example.com/tcp/4001/p2p/...".to_string(),
        "/ip4/relay2.example.com/tcp/4001/p2p/...".to_string(),
    ],
    ..Default::default()
};

let mut swarm = create_swarm(keypair, config)?;

// Connect to relay servers
connect_to_relay_servers(&mut swarm, &config.relay_servers)?;

// Listen for incoming connections
swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
```

## Security

### Transport Encryption

**Noise Protocol**:
- XX handshake pattern
- ChaCha20-Poly1305 encryption
- Forward secrecy
- Mutual authentication

```rust
// Noise encryption is automatic in libp2p
transport
    .upgrade(upgrade::Version::V1)
    .authenticate(noise::Config::new(&keypair)?)
    .multiplex(yamux::Config::default())
```

### Device Authorization

Only authorized devices can sync:

```rust
// Check device authorization before syncing
fn verify_device_authorized(
    conn: &Connection,
    peer_device_id: Uuid,
    local_user_id: Uuid,
) -> Result<bool> {
    let device = get_device(conn, peer_device_id)?;
    Ok(device.user_id == local_user_id)
}
```

### Message Signing

Future enhancement for message authenticity:

```rust
// Sign operation with device key
let signature = device_key.sign(&operation_data);

let signed_op = SignedOplogEntry {
    operation,
    signature,
    device_public_key,
};

// Verify on receiving end
if !verify_signature(&signed_op) {
    return Err("Invalid signature");
}
```

## Performance Optimization

### Batching

Send multiple operations in one message:

```rust
const BATCH_SIZE: usize = 100;

let mut batch = Vec::new();
for op in pending_ops {
    batch.push(op);

    if batch.len() >= BATCH_SIZE {
        send_ops(&mut swarm, &batch)?;
        batch.clear();
    }
}

// Send remaining
if !batch.is_empty() {
    send_ops(&mut swarm, &batch)?;
}
```

### Delta Sync

Only send operations since last sync:

```rust
// Store last sync timestamp per peer
let last_sync = get_last_sync_timestamp(conn, peer_id)?;

// Request only new operations
let request = SyncMessage::RequestOps {
    since: last_sync,
};

send_message(&mut swarm, peer_id, request)?;
```

### Compression

Compress large operation payloads:

```rust
use flate2::write::GzEncoder;

fn compress_ops(ops: &[OplogEntry]) -> Result<Vec<u8>> {
    let json = serde_json::to_vec(ops)?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&json)?;
    Ok(encoder.finish()?)
}
```

## Monitoring & Debugging

### Logging

Enable detailed P2P logs:

```bash
RUST_LOG=ahenk::logic::sync=debug ahenk-cli start
```

### Metrics

Track sync statistics:

```rust
pub struct SyncStats {
    pub connected_peers: usize,
    pub total_ops_sent: u64,
    pub total_ops_received: u64,
    pub last_sync: DateTime<Utc>,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}
```

### Network Events

Monitor libp2p events:

```rust
while let Some(event) = swarm.select_next_some().await {
    match event {
        SwarmEvent::NewListenAddr { address, .. } => {
            println!("Listening on {}", address);
        }
        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
            println!("Connected to {}", peer_id);
        }
        SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
            println!("Disconnected from {}: {:?}", peer_id, cause);
        }
        // ... handle other events
    }
}
```

## Troubleshooting

### Peers can't discover each other

**Check**:
1. mDNS enabled: `config.enable_mdns = true`
2. Same local network
3. Firewall allows multicast
4. Same user account on both devices

**Solution**:
```bash
# Try relay servers instead
ahenk-cli config set network.relay_servers "/ip4/relay.example.com/tcp/4001/p2p/..."
```

### High latency

**Possible causes**:
- Using relay instead of direct connection
- Geographic distance between peers
- Network congestion

**Check**:
```bash
# See if using relay or direct
ahenk-cli peer list --verbose
```

### Sync conflicts

**Check operation logs**:
```sql
SELECT * FROM oplog WHERE timestamp > ?
ORDER BY timestamp DESC;
```

**Verify HLC ordering**:
```rust
let ops = get_all_ops(conn)?;
for i in 1..ops.len() {
    assert!(ops[i].timestamp >= ops[i-1].timestamp);
}
```

## Best Practices

1. **Always verify device authorization** before syncing
2. **Use batching** for multiple operations
3. **Enable both mDNS and relay** for reliability
4. **Monitor sync status** in production
5. **Implement retry logic** for failed syncs
6. **Log sync events** for debugging
7. **Test with network partitions** to ensure eventual consistency

## See Also

- [AHENK_INFRASTRUCTURE_GUIDE.md](../AHENK_INFRASTRUCTURE_GUIDE.md) - Complete architecture
- [README.md](../README.md) - Getting started
- [CLI_USAGE.md](CLI_USAGE.md) - CLI sync commands
- [libp2p Documentation](https://docs.libp2p.io/) - libp2p details
