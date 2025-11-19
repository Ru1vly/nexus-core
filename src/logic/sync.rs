use crate::crdt;
use crate::db::operations;
use crate::models::{OplogEntry, Peer};
use chrono::Utc;
use libp2p::gossipsub::{MessageAuthenticity, ValidationMode};
use libp2p::{
    core::upgrade, dcutr, gossipsub, identity, mdns, multiaddr::Protocol, noise, relay,
    swarm::NetworkBehaviour, tcp, yamux, PeerId, Swarm, Transport,
};
use rusqlite::Connection;
use std::time::Duration;
use uuid::Uuid;

/// Network behavior combining mDNS, Gossipsub, Relay, and DCUtR
#[derive(NetworkBehaviour)]
pub struct AhenkBehaviour {
    /// mDNS for local network peer discovery
    pub mdns: mdns::tokio::Behaviour,
    /// Gossipsub for message propagation
    pub gossipsub: gossipsub::Behaviour,
    /// Relay client for NAT traversal
    pub relay_client: relay::client::Behaviour,
    /// Direct Connection Upgrade through Relay (DCUtR)
    pub dcutr: dcutr::Behaviour,
}

/// Configuration for P2P network
#[derive(Debug, Clone)]
pub struct P2PConfig {
    /// Enable mDNS for local discovery
    pub enable_mdns: bool,
    /// Enable relay for NAT traversal
    pub enable_relay: bool,
    /// Bootstrap nodes to connect to
    pub bootstrap_nodes: Vec<String>,
    /// Relay server addresses
    pub relay_servers: Vec<String>,
    /// Gossipsub heartbeat interval
    pub heartbeat_interval: Duration,
    /// Maximum message size for gossipsub
    pub max_message_size: usize,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            enable_mdns: true,
            enable_relay: true,
            bootstrap_nodes: vec![],
            relay_servers: vec![],
            heartbeat_interval: Duration::from_secs(10),
            max_message_size: 65536, // 64KB
        }
    }
}

/// Message types for P2P communication
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum SyncMessage {
    /// Request oplog entries since a specific timestamp
    RequestSync { user_id: Uuid, since_timestamp: i64 },
    /// Response with oplog entries
    SyncData {
        user_id: Uuid,
        entries: Vec<OplogEntry>,
    },
    /// Announce presence with device info
    Announce {
        user_id: Uuid,
        device_id: Uuid,
        peer_id: String,
    },
    /// Ping message for keepalive
    Ping { timestamp: i64 },
    /// Pong response to ping
    Pong { timestamp: i64 },
}

pub fn handle_sync_message(
    conn: &mut Connection,
    msg: SyncMessage,
) -> Result<Option<SyncMessage>, String> {
    match msg {
        SyncMessage::RequestSync {
            user_id,
            since_timestamp,
        } => {
            // Note: get_oplog_entries_since only takes (conn, since_timestamp)
            // It returns all entries after the timestamp, regardless of user_id
            let entries = operations::get_oplog_entries_since(conn, since_timestamp)
                .map_err(|e| e.to_string())?;
            Ok(Some(SyncMessage::SyncData { user_id, entries }))
        }
        SyncMessage::SyncData {
            user_id: _,
            entries,
        } => {
            crdt::merge(conn, &entries).map_err(|e| e.to_string())?;
            Ok(None)
        }
        SyncMessage::Announce {
            user_id,
            device_id,
            peer_id,
        } => {
            update_peer_info(conn, user_id, device_id, peer_id, None)?;
            Ok(None)
        }
        SyncMessage::Ping { .. } => Ok(None),
        SyncMessage::Pong { .. } => Ok(None),
    }
}

/// Generate a unique device ID and keypair for P2P communication
pub fn generate_device_id() -> (PeerId, identity::Keypair) {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    (local_peer_id, local_key)
}

/// Create a new P2P network swarm for local network synchronization with relay support
pub fn create_swarm(
    keypair: identity::Keypair,
    config: P2PConfig,
) -> Result<Swarm<AhenkBehaviour>, Box<dyn std::error::Error>> {
    let peer_id = PeerId::from(keypair.public());

    // Create a Gossipsub topic for sync messages
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(config.heartbeat_interval)
        .validation_mode(ValidationMode::Strict)
        .max_transmit_size(config.max_message_size)
        .duplicate_cache_time(Duration::from_secs(60))
        .build()
        .map_err(std::io::Error::other)?;

    // Build a Gossipsub behaviour
    let mut gossipsub = gossipsub::Behaviour::new(
        MessageAuthenticity::Signed(keypair.clone()),
        gossipsub_config,
    )
    .map_err(std::io::Error::other)?;

    // Create a Gossipsub topic
    let topic = gossipsub::IdentTopic::new("nexus-sync");
    gossipsub.subscribe(&topic)?;

    // Create mDNS behaviour for local network discovery (if enabled)
    let mdns = if config.enable_mdns {
        mdns::tokio::Behaviour::new(
            mdns::Config {
                ttl: Duration::from_secs(60 * 6), // 6 minutes
                query_interval: Duration::from_secs(60),
                ..Default::default()
            },
            peer_id,
        )?
    } else {
        mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?
    };

    // Create relay client for NAT traversal
    let (_, relay_client) = relay::client::new(peer_id);

    // Create DCUtR behaviour for hole punching
    let dcutr = dcutr::Behaviour::new(peer_id);

    let behaviour = AhenkBehaviour {
        mdns,
        gossipsub,
        relay_client,
        dcutr,
    };

    // Build the transport using the tokio API
    let tcp_transport = tcp::tokio::Transport::default();
    let transport = tcp_transport
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::Config::new(&keypair)?)
        .multiplex(yamux::Config::default())
        .boxed();

    // Use the libp2p 0.56 API with tokio executor
    let swarm = Swarm::new(
        transport,
        behaviour,
        peer_id,
        libp2p::swarm::Config::with_tokio_executor(),
    );

    Ok(swarm)
}

/// Create a swarm with default configuration
pub fn create_swarm_default(
    keypair: identity::Keypair,
) -> Result<Swarm<AhenkBehaviour>, Box<dyn std::error::Error>> {
    create_swarm(keypair, P2PConfig::default())
}

/// Parse a multiaddr string and return the PeerId if present
pub fn parse_multiaddr_peer_id(addr: &str) -> Option<PeerId> {
    use libp2p::multiaddr::Multiaddr;

    let multiaddr: Multiaddr = addr.parse().ok()?;

    for protocol in multiaddr.iter() {
        if let Protocol::P2p(peer_id) = protocol {
            return Some(peer_id);
        }
    }
    None
}

/// Connect to bootstrap nodes
pub fn connect_to_bootstrap_nodes(
    swarm: &mut Swarm<AhenkBehaviour>,
    bootstrap_nodes: &[String],
) -> Result<usize, String> {
    use libp2p::multiaddr::Multiaddr;

    let mut connected = 0;

    for node_addr in bootstrap_nodes {
        match node_addr.parse::<Multiaddr>() {
            Ok(addr) => match swarm.dial(addr.clone()) {
                Ok(_) => {
                    connected += 1;
                    println!("Dialing bootstrap node: {}", node_addr);
                }
                Err(e) => {
                    eprintln!("Failed to dial bootstrap node {}: {:?}", node_addr, e);
                }
            },
            Err(e) => {
                eprintln!("Invalid multiaddr {}: {:?}", node_addr, e);
            }
        }
    }

    Ok(connected)
}

/// Connect to relay servers for NAT traversal
pub fn connect_to_relay_servers(
    swarm: &mut Swarm<AhenkBehaviour>,
    relay_servers: &[String],
) -> Result<usize, String> {
    use libp2p::multiaddr::Multiaddr;

    let mut connected = 0;

    for relay_addr in relay_servers {
        match relay_addr.parse::<Multiaddr>() {
            Ok(addr) => match swarm.dial(addr.clone()) {
                Ok(_) => {
                    connected += 1;
                    println!("Dialing relay server: {}", relay_addr);
                }
                Err(e) => {
                    eprintln!("Failed to dial relay server {}: {:?}", relay_addr, e);
                }
            },
            Err(e) => {
                eprintln!("Invalid relay multiaddr {}: {:?}", relay_addr, e);
            }
        }
    }

    Ok(connected)
}

/// Update peer information in the database
pub fn update_peer_info(
    conn: &Connection,
    user_id: Uuid,
    device_id: Uuid,
    _peer_id: String,
    ip_address: Option<String>,
) -> Result<(), String> {
    // Check if peer exists
    let peers = operations::get_peers_by_user_id(conn, user_id)
        .map_err(|e| format!("Failed to get peers: {}", e))?;

    let peer_exists = peers.iter().any(|p| p.device_id == device_id);

    if !peer_exists {
        let new_peer = Peer {
            peer_id: Uuid::new_v4(),
            user_id,
            device_id,
            last_known_ip: ip_address,
            last_sync_time: Some(Utc::now().timestamp()),
        };
        operations::create_peer(conn, &new_peer)
            .map_err(|e| format!("Failed to create peer: {}", e))?;
    }

    Ok(())
}

/// Encode a sync message to bytes for transmission
pub fn encode_sync_message(message: &SyncMessage) -> Result<Vec<u8>, String> {
    serde_json::to_vec(message).map_err(|e| format!("Failed to encode message: {}", e))
}

/// Decode a sync message from bytes
pub fn decode_sync_message(bytes: &[u8]) -> Result<SyncMessage, String> {
    serde_json::from_slice(bytes).map_err(|e| format!("Failed to decode message: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_device_id() {
        let (peer_id, keypair) = generate_device_id();
        let derived_peer_id = PeerId::from(keypair.public());
        assert_eq!(peer_id, derived_peer_id);
    }

    #[test]
    fn test_p2p_config_default() {
        let config = P2PConfig::default();
        assert!(config.enable_mdns);
        assert!(config.enable_relay);
        assert_eq!(config.heartbeat_interval, Duration::from_secs(10));
    }

    #[test]
    fn test_encode_decode_sync_message() {
        let msg = SyncMessage::Ping {
            timestamp: Utc::now().timestamp(),
        };
        let encoded = encode_sync_message(&msg).unwrap();
        let decoded = decode_sync_message(&encoded).unwrap();

        match (msg, decoded) {
            (SyncMessage::Ping { .. }, SyncMessage::Ping { .. }) => (),
            _ => panic!("Message type mismatch"),
        }
    }

    #[test]
    fn test_parse_multiaddr_peer_id() {
        let addr =
            "/ip4/127.0.0.1/tcp/4001/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN";
        let peer_id = parse_multiaddr_peer_id(addr);
        assert!(peer_id.is_some());
    }
}
