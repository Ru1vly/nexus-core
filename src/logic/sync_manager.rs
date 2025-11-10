use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use libp2p::PeerId;
#[cfg(feature = "tauri-api")]
use tauri::AppHandle;
use crate::models::OplogEntry;
use libp2p::{Swarm, gossipsub, identity, mdns};
use libp2p::swarm::SwarmEvent;
use crate::logic::sync::{NexusBehaviour, NexusBehaviourEvent, P2PConfig, create_swarm, connect_to_bootstrap_nodes, connect_to_relay_servers, SyncMessage, encode_sync_message};

/// Sync manager for handling P2P network events and synchronization
pub struct SyncManager {
    /// The libp2p swarm
    swarm: Swarm<NexusBehaviour>,
    /// User ID for this device
    user_id: Uuid,
    /// Device ID for this device
    device_id: Uuid,
    /// Database connection (thread-safe)
    conn: Arc<Mutex<Connection>>,
    /// Gossipsub topic for sync messages
    topic: gossipsub::IdentTopic,
    /// Is the manager currently actively syncing/connected to peers
    is_syncing: bool,
    /// Timestamp of the last successful sync operation
    last_sync_time: Option<DateTime<Utc>>,
    /// List of currently connected peer IDs
    connected_peers: Vec<PeerId>,
    /// Tauri AppHandle to emit events to the frontend (only when compiled with `tauri-api`)
    #[cfg(feature = "tauri-api")]
    app_handle: AppHandle,
    /// Queue of pending oplog entries to be synced when coming back online
    pending_changes: VecDeque<OplogEntry>,
    /// Is the device currently online
    is_online: bool,
}

impl SyncManager {
    /// Create a new sync manager
    #[cfg(feature = "tauri-api")]
    pub fn new(
        keypair: identity::Keypair,
        user_id: Uuid,
        device_id: Uuid,
        conn: Arc<Mutex<Connection>>,
        config: P2PConfig,
        app_handle: AppHandle,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let swarm = create_swarm(keypair, config)?;
        let topic = gossipsub::IdentTopic::new("nexus-sync");

        Ok(Self {
            swarm,
            user_id,
            device_id,
            conn,
            topic,
            is_syncing: false,
            last_sync_time: None,
            pending_changes: VecDeque::new(),
            is_online: true,
            connected_peers: Vec::new(),
            app_handle,
        })
    }

    #[cfg(not(feature = "tauri-api"))]
    pub fn new(
        keypair: identity::Keypair,
        user_id: Uuid,
        device_id: Uuid,
        conn: Arc<Mutex<Connection>>,
        config: P2PConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let swarm = create_swarm(keypair, config)?;
        let topic = gossipsub::IdentTopic::new("nexus-sync");

        Ok(Self {
            swarm,
            user_id,
            device_id,
            conn,
            topic,
            is_syncing: false,
            last_sync_time: None,
            pending_changes: VecDeque::new(),
            is_online: true,
            connected_peers: Vec::new(),
        })
    }

    /// Start listening on all network interfaces
    pub fn listen(&mut self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let listen_addr = format!("/ip4/0.0.0.0/tcp/{}", port);
        self.swarm.listen_on(listen_addr.parse()?)?;
        Ok(())
    }

    /// Connect to bootstrap and relay nodes
    pub fn connect_to_network(
        &mut self,
        bootstrap_nodes: &[String],
        relay_servers: &[String],
    ) -> Result<(), String> {
        if !bootstrap_nodes.is_empty() {
            connect_to_bootstrap_nodes(&mut self.swarm, bootstrap_nodes)?;
        }

        if !relay_servers.is_empty() {
            connect_to_relay_servers(&mut self.swarm, relay_servers)?;
        }

        Ok(())
    }

    /// Broadcast an announce message to the network
    pub fn announce_presence(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let peer_id = *self.swarm.local_peer_id();
        let message = SyncMessage::Announce {
            user_id: self.user_id,
            device_id: self.device_id,
            peer_id: peer_id.to_string(),
        };

        let encoded = encode_sync_message(&message).map_err(|e| std::io::Error::other(e))?;

        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.topic.clone(), encoded)
            .map_err(|e| std::io::Error::other(format!("Failed to publish: {:?}", e)))?;

        Ok(())
    }

    /// Request sync from peers
    pub fn request_sync(
        &mut self,
        since_timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let message = SyncMessage::RequestSync {
            user_id: self.user_id,
            since_timestamp: since_timestamp.timestamp(),
        };

        let encoded = encode_sync_message(&message).map_err(|e| std::io::Error::other(e))?;

        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.topic.clone(), encoded)
            .map_err(|e| std::io::Error::other(format!("Failed to publish: {:?}", e)))?;

        Ok(())
    }

    /// Send sync data to peers
    pub fn send_sync_data(
        &mut self,
        entries: Vec<crate::models::OplogEntry>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let message = SyncMessage::SyncData {
            user_id: self.user_id,
            entries,
        };

        let encoded = encode_sync_message(&message).map_err(|e| std::io::Error::other(e))?;

        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.topic.clone(), encoded)
            .map_err(|e| std::io::Error::other(format!("Failed to publish: {:?}", e)))?;

        Ok(())
    }

    /// Get the current syncing status
    pub fn get_is_syncing(&self) -> bool {
        self.is_syncing
    }

    /// Get the last sync time
    pub fn get_last_sync_time(&self) -> Option<DateTime<Utc>> {
        self.last_sync_time
    }

    /// Get the list of connected peers
    pub fn get_connected_peers(&self) -> Vec<String> {
        self.connected_peers.iter().map(|p| p.to_string()).collect()
    }

    #[cfg(feature = "tauri-api")]
    fn emit_sync_status(&self) {
        let status = serde_json::json!({
            "is_syncing": self.is_syncing,
            "last_sync_time": self.last_sync_time.map(|dt| dt.to_rfc3339()),
            "connected_peers": self.connected_peers.iter().map(|p| p.to_string()).collect::<Vec<String>>(),
        });
        let _ = self.app_handle.emit("sync-status-update", status);

        // Also emit pending changes count
        self.emit_pending_changes();
    }

    #[cfg(not(feature = "tauri-api"))]
    fn emit_sync_status(&self) {
        // No-op when tauri feature is not enabled
        let _ = (&self.connected_peers);
    }

    #[cfg(feature = "tauri-api")]
    fn emit_pending_changes(&self) {
        let pending_status = serde_json::json!({
            "pendingCount": self.pending_changes.len(),
        });
        let _ = self.app_handle.emit("pending-changes-update", pending_status);
    }

    #[cfg(not(feature = "tauri-api"))]
    fn emit_pending_changes(&self) {
        // No-op when tauri feature is not enabled
        let _ = self.pending_changes.len();
    }
    
    /// Add a change to the pending queue when offline
    pub fn add_pending_change(&mut self, entry: OplogEntry) {
        self.pending_changes.push_back(entry);
        self.emit_pending_changes();
    }
    
    /// Set the online status of the sync manager
    pub fn set_online_status(&mut self, is_online: bool) -> Result<(), Box<dyn std::error::Error>> {
        let was_offline = !self.is_online;
        self.is_online = is_online;
        
        // If we just came back online and have pending changes, sync them
        if is_online && was_offline && !self.pending_changes.is_empty() {
            self.sync_pending_changes()?;
        }
        
        self.emit_sync_status();
        Ok(())
    }
    
    /// Sync any pending changes that accumulated while offline
    pub fn sync_pending_changes(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.is_online || self.pending_changes.is_empty() {
            return Ok(());
        }
        
        // Convert queue to vector for sending
        let pending_entries: Vec<OplogEntry> = self.pending_changes.drain(..).collect();
        
        // If no peers are connected, we can't sync yet - return the entries to the queue
        if self.connected_peers.is_empty() {
            for entry in pending_entries {
                self.pending_changes.push_back(entry);
            }
            return Ok(());
        }
        
        // Send the pending changes
        self.send_sync_data(pending_entries)?;
        self.emit_pending_changes();
        
        Ok(())
    }
    
    /// Get the number of pending changes
    pub fn get_pending_changes_count(&self) -> usize {
        self.pending_changes.len()
    }

    /// Process a single network event (non-blocking)
    pub async fn process_event(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        use futures::StreamExt;

        match self.swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on: {}", address);
            }
            SwarmEvent::Behaviour(event) => {
                self.handle_behaviour_event(event)?;
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!("Connected to peer: {}", peer_id);
                self.connected_peers.push(peer_id);
                self.is_syncing = true;
                self.emit_sync_status();
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                println!("Connection closed with {}: {:?}", peer_id, cause);
                self.connected_peers.retain(|p| p != &peer_id);
                if self.connected_peers.is_empty() {
                    self.is_syncing = false;
                }
                self.emit_sync_status();
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle behaviour-specific events
    fn handle_behaviour_event(
        &mut self,
        event: <NexusBehaviour as libp2p::swarm::NetworkBehaviour>::ToSwarm,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            NexusBehaviourEvent::Mdns(mdns::Event::Discovered(peers)) => {
                for (peer_id, _addr) in peers {
                    println!("Discovered peer: {}", peer_id);
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .add_explicit_peer(&peer_id);
                }
            }
            NexusBehaviourEvent::Mdns(mdns::Event::Expired(peers)) => {
                for (peer_id, _addr) in peers {
                    println!("Peer expired: {}", peer_id);
                    self.swarm
                        .behaviour_mut()
                        .gossipsub
                        .remove_explicit_peer(&peer_id);
                }
            }
            NexusBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. }) => {
                self.handle_gossipsub_message(message)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle a gossipsub message
    fn handle_gossipsub_message(
        &mut self,
        message: gossipsub::Message,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let sync_message = crate::logic::sync::decode_sync_message(&message.data)?;
        match sync_message {
            SyncMessage::SyncData { .. } => {
                self.last_sync_time = Some(Utc::now());
                self.emit_sync_status();
            }
            _ => {}
        }
        Ok(())
    }

    /// Run the event loop indefinitely
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            self.process_event().await?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::sync::generate_device_id;

    #[test]
    fn test_sync_manager_creation() {
        use rusqlite::Connection;

        let conn = Connection::open_in_memory().unwrap();
        let conn = Arc::new(Mutex::new(conn));
        let (_, keypair) = generate_device_id();
        let user_id = Uuid::new_v4();
        let device_id = Uuid::new_v4();
        let config = P2PConfig::default();

        let manager = SyncManager::new(keypair, user_id, device_id, conn, config);
        assert!(manager.is_ok());
    }
}
