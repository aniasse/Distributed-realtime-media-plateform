use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use log::{info, error, debug, warn};

use crate::shared::domain::{Room, Peer, Track, MediaKind, RoomId, PeerId, TrackId, RoomState, PublisherId};
use crate::shared::media::{RTPPacket, RTCPPacket, Transport, TransportError, ForwardingStrategy, PacketProcessor, PacketError};
use crate::shared::utils::{Logger, Metrics, ErrorHandler};

pub struct SFU {
    pub rooms: Arc<RwLock<HashMap<Uuid, Room>>>,
    pub peers: Arc<RwLock<HashMap<Uuid, Peer>>>,
    pub tracks: Arc<RwLock<HashMap<Uuid, Track>>>,
    pub packet_processor: Box<dyn PacketProcessor>,
    pub logger: Logger,
    pub metrics: Metrics,
    pub error_handler: ErrorHandler,
}

impl SFU {
    pub fn new(packet_processor: Box<dyn PacketProcessor>) -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            peers: Arc::new(RwLock::new(HashMap::new())),
            tracks: Arc::new(RwLock::new(HashMap::new())),
            packet_processor,
            logger: Logger::new("sfu"),
            metrics: Metrics::new(),
            error_handler: ErrorHandler::new("sfu"),
        }
    }

    pub async fn create_room(&self, tenant_id: Uuid, max_participants: u32) -> Result<RoomId, RoomError> {
        self.logger.info("Creating new room");
        
        let room_id = Uuid::new_v4();
        let room = Room {
            id: room_id,
            tenant_id,
            peers: HashMap::new(),
            tracks: HashMap::new(),
            max_participants,
            created_at: chrono::Utc::now(),
            state: RoomState::Active,
        };
        
        self.rooms.write().await.insert(room_id, room);
        self.metrics.increment_counter("rooms_created", 1);
        
        Ok(RoomId(room_id))
    }

    pub async fn delete_room(&self, room_id: Uuid) -> Result<(), RoomError> {
        self.logger.info(&format!("Deleting room {}", room_id));
        
        if self.rooms.write().await.remove(&room_id).is_some() {
            self.metrics.increment_counter("rooms_deleted", 1);
            Ok(())
        } else {
            Err(RoomError::RoomNotFound)
        }
    }

    pub async fn add_peer(&self, room_id: Uuid, peer_id: Uuid) -> Result<(), RoomError> {
        self.logger.info(&format!("Adding peer {} to room {}", peer_id, room_id));
        
        let mut rooms = self.rooms.write().await;
        if let Some(room) = rooms.get_mut(&room_id) {
            if room.peers.len() as u32 >= room.max_participants {
                return Err(RoomError::MaxParticipantsReached);
            }
            
            let peer = Peer {
                id: peer_id,
                connected_at: chrono::Utc::now(),
                tracks_subscribed: HashSet::new(),
                bandwidth_estimate: crate::shared::domain::BandwidthEstimate {
                    available_upload: 10000,
                    available_download: 10000,
                    current_usage: 0,
                },
                connection_state: crate::shared::domain::ConnectionState::Connected,
                ice_candidates: Vec::new(),
                dtls_fingerprints: Vec::new(),
            };
            
            room.peers.insert(peer_id, peer);
            self.metrics.increment_counter("peers_connected", 1);
            Ok(())
        } else {
            Err(RoomError::RoomNotFound)
        }
    }

    pub async fn remove_peer(&self, room_id: Uuid, peer_id: Uuid) -> Result<(), RoomError> {
        self.logger.info(&format!("Removing peer {} from room {}", peer_id, room_id));
        
        let mut rooms = self.rooms.write().await;
        if let Some(room) = rooms.get_mut(&room_id) {
            if room.peers.remove(&peer_id).is_some() {
                self.metrics.increment_counter("peers_disconnected", 1);
                
                // Remove peer's tracks
                let mut tracks = self.tracks.write().await;
                tracks.retain(|_, track| track.publisher_id.peer_id != peer_id);
                
                Ok(())
            } else {
                Err(RoomError::PeerNotFound)
            }
        } else {
            Err(RoomError::RoomNotFound)
        }
    }

    pub async fn add_track(&self, room_id: Uuid, track: Track) -> Result<(), RoomError> {
        self.logger.info(&format!("Adding track {} to room {}", track.id, room_id));
        
        let mut rooms = self.rooms.write().await;
        if let Some(room) = rooms.get_mut(&room_id) {
            room.tracks.insert(track.id, track.clone());
            
            // Add to global tracks
            self.tracks.write().await.insert(track.id, track);
            
            self.metrics.increment_counter("tracks_added", 1);
            Ok(())
        } else {
            Err(RoomError::RoomNotFound)
        }
    }

    pub async fn handle_packet(&self, packet: RTPPacket, room_id: Uuid) -> Result<(), PacketError> {
        self.logger.debug(&format!("Handling RTP packet in room {}", room_id));
        
        // Find the room and track
        let rooms = self.rooms.read().await;
        if let Some(room) = rooms.get(&room_id) {
            // Find the track
            let tracks = self.tracks.read().await;
            if let Some(track) = tracks.values().find(|t| t.ssrc == packet.ssrc) {
                // Get forwarding strategy
                let strategy = self.packet_processor.get_forwarding_strategy(track.id);
                
                match strategy {
                    ForwardingStrategy::Unicast { peer_ids } => {
                        self.forward_to_peers(packet, peer_ids).await?;
                    }
                    ForwardingStrategy::Multicast { group_id } => {
                        // Implement multicast forwarding
                        self.forward_to_group(packet, group_id).await?;
                    }
                    ForwardingStrategy::Simulcast { layers } => {
                        // Implement simulcast forwarding
                        self.forward_simulcast(packet, layers).await?;
                    }
                    ForwardingStrategy::SVC { layers } => {
                        // Implement SVC forwarding
                        self.forward_svc(packet, layers).await?;
                    }
                }
                
                self.metrics.increment_counter("packets_forwarded", 1);
                Ok(())
            } else {
                Err(PacketError::InvalidPacket)
            }
        } else {
            Err(PacketError::InvalidPacket)
        }
    }

    async fn forward_to_peers(&self, packet: RTPPacket, peer_ids: Vec<Uuid>) -> Result<(), PacketError> {
        // Forward packet to specific peers
        for peer_id in peer_ids {
            // Get peer connection info
            let peers = self.peers.read().await;
            if let Some(peer) = peers.get(&peer_id) {
                // Simulate forwarding (in real implementation, this would send over network)
                self.logger.debug(&format!("Forwarding packet to peer {}", peer_id));
            }
        }
        Ok(())
    }

    async fn forward_to_group(&self, _packet: RTPPacket, _group_id: Uuid) -> Result<(), PacketError> {
        // Implement multicast forwarding
        Ok(())
    }

    async fn forward_simulcast(&self, _packet: RTPPacket, _layers: Vec<Uuid>) -> Result<(), PacketError> {
        // Implement simulcast forwarding
        Ok(())
    }

    async fn forward_svc(&self, _packet: RTPPacket, _layers: Vec<Uuid>) -> Result<(), PacketError> {
        // Implement SVC forwarding
        Ok(())
    }

    pub async fn handle_rtcp(&self, packet: RTCPPacket, peer_id: Uuid) -> Result<(), PacketError> {
        self.logger.debug(&format!("Handling RTCP packet from peer {}", peer_id));
        
        // Process RTCP feedback
        match packet.packet_type {
            crate::shared::media::RTCPPacketType::RTPFB => {
                // Handle NACK
                if let Ok(nack) = serde_json::from_slice::<crate::shared::media::NACK>(&packet.payload) {
                    self.handle_nack(nack, peer_id).await?;
                }
            }
            crate::shared::media::RTCPPacketType::PSFB => {
                // Handle PLI, FIR, SLI
                match packet.report_count {
                    1 => { /* PLI */ }
                    4 => { /* FIR */ }
                    6 => { /* SLI */ }
                    _ => {}
                }
            }
            _ => {}
        }
        
        Ok(())
    }

    async fn handle_nack(&self, nack: crate::shared::media::NACK, peer_id: Uuid) -> Result<(), PacketError> {
        self.logger.debug(&format!("Handling NACK from peer {}: pid={} blp={}", peer_id, nack.pid, nack.blp));
        
        // Implement NACK handling (retransmit missing packets)
        Ok(())
    }

    pub async fn get_room_stats(&self, room_id: Uuid) -> Result<RoomStats, RoomError> {
        let rooms = self.rooms.read().await;
        if let Some(room) = rooms.get(&room_id) {
            let peers = self.peers.read().await;
            let peer_count = room.peers.len();
            let active_peers = room.peers.values().filter(|p| p.connection_state == crate::shared::domain::ConnectionState::Connected).count();
            let track_count = room.tracks.len();
            
            Ok(RoomStats {
                room_id,
                peer_count: peer_count as u32,
                active_peers: active_peers as u32,
                track_count: track_count as u32,
                max_participants: room.max_participants,
                created_at: room.created_at,
            })
        } else {
            Err(RoomError::RoomNotFound)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomStats {
    pub room_id: Uuid,
    pub peer_count: u32,
    pub active_peers: u32,
    pub track_count: u32,
    pub max_participants: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub enum RoomError {
    RoomNotFound,
    PeerNotFound,
    MaxParticipantsReached,
    InvalidTrack,
    InternalError,
}

impl std::fmt::Display for RoomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for RoomError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_sfu_creation() {
        let packet_processor = MockPacketProcessor::new();
        let sfu = SFU::new(packet_processor);
        
        assert!(sfu.rooms.read().await.is_empty());
        assert!(sfu.peers.read().await.is_empty());
        assert!(sfu.tracks.read().await.is_empty());
    }
    
    #[tokio::test]
    async fn test_room_creation() {
        let packet_processor = MockPacketProcessor::new();
        let sfu = SFU::new(packet_processor);
        
        let tenant_id = Uuid::new_v4();
        let room_id = sfu.create_room(tenant_id, 10).await.unwrap();
        
        assert!(sfu.rooms.read().await.contains_key(&room_id.0));
        assert_eq!(sfu.metrics.get_counter("rooms_created").unwrap(), 1);
    }
    
    struct MockPacketProcessor;
    impl MockPacketProcessor {
        fn new() -> Self { Self }
    }
    
    impl PacketProcessor for MockPacketProcessor {
        fn process_rtp(&self, _packet: RTPPacket) -> Result<(), PacketError> {
            Ok(())
        }
        fn process_rtcp(&self, _packet: RTCPPacket) -> Result<(), PacketError> {
            Ok(())
        }
        fn get_forwarding_strategy(&self, _track_id: Uuid) -> ForwardingStrategy {
            ForwardingStrategy::Unicast { peer_ids: vec![] }
        }
    }
}