use std::collections::{HashMap};
use std::sync::Arc;
use tokio::sync::{Mutex};
use tokio::task::JoinHandle;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use dashmap::DashMap;
use actix::Addr;
use sqlx::PgPool;

// WebRTC imports
use webrtc::api::API;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::setting_engine::SettingEngine;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::ice_transport::ice_candidate_type::ICECandidateType;

pub mod signaling;
use crate::signaling::{SignalingSession};

pub use shared::domain::{Room, Peer, Track, RoomId, RoomState, ConnectionState};
pub use shared::media::{RTPPacket, PacketProcessor, PacketError, Transport, ForwardingStrategy};
use shared::utils::{Logger, Metrics, ErrorHandler};

pub struct SFU {
    pub db_pool: PgPool,
    pub webrtc_api: Arc<API>,
    pub rooms: Arc<DashMap<Uuid, Room>>,
    pub peers: Arc<DashMap<Uuid, Peer>>,
    pub relay_tracks: Arc<DashMap<Uuid, Arc<TrackLocalStaticRTP>>>,
    pub room_sessions: Arc<DashMap<Uuid, Vec<Addr<SignalingSession>>>>,
    pub sessions: Arc<DashMap<Uuid, Addr<SignalingSession>>>,
    pub packet_processor: Arc<Box<dyn PacketProcessor>>,
    pub logger: Logger,
    pub metrics: Arc<Mutex<Metrics>>,
    pub error_handler: ErrorHandler,
    pub worker_handles: Arc<Mutex<HashMap<Uuid, JoinHandle<()>>>>,
}

impl SFU {
    pub fn new(db_pool: PgPool, packet_processor: Box<dyn PacketProcessor>) -> Self {
        let mut m = MediaEngine::default();
        let _ = m.register_default_codecs();
        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut m).expect("Failed interceptors");
        
        let mut s = SettingEngine::default();
        
        // --- CONFIGURATION NAT 1:1 VITALE POUR DOCKER ---
        // On force le SFU à annoncer 127.0.0.1 pour les connexions locales
        s.set_nat_1to1_ips(vec!["127.0.0.1".to_string()], ICECandidateType::Host);
        
        // Tentative de fixer la plage de ports UDP avec la signature correcte
        let _ = s.set_ice_gatherer_port_range(10000, 10010);

        let api = webrtc::api::APIBuilder::new()
            .with_media_engine(m)
            .with_interceptor_registry(registry)
            .with_setting_engine(s)
            .build();

        Self {
            db_pool,
            webrtc_api: Arc::new(api),
            rooms: Arc::new(DashMap::new()),
            peers: Arc::new(DashMap::new()),
            relay_tracks: Arc::new(DashMap::new()),
            room_sessions: Arc::new(DashMap::new()),
            sessions: Arc::new(DashMap::new()),
            packet_processor: Arc::new(packet_processor),
            logger: Logger::new("sfu"),
            metrics: Arc::new(Mutex::new(Metrics::new())),
            error_handler: ErrorHandler::new("sfu"),
            worker_handles: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create_webrtc_pc(&self) -> Result<RTCPeerConnection, webrtc::Error> {
        let config = RTCConfiguration {
            ice_servers: vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                ..Default::default()
            }],
            ..Default::default()
        };
        self.webrtc_api.new_peer_connection(config).await
    }

    pub async fn register_session(&self, room_id: Uuid, addr: Addr<SignalingSession>) {
        let mut sessions = self.room_sessions.entry(room_id).or_insert(Vec::new());
        sessions.push(addr);
    }

    pub async fn check_room_exists(&self, room_id: Uuid) -> bool {
        let result = sqlx::query("SELECT id FROM rooms WHERE id = $1")
            .bind(room_id)
            .fetch_optional(&self.db_pool)
            .await;
        result.is_ok()
    }
    
    pub async fn create_room(&self, tenant_id: Uuid, max_participants: u32) -> Result<RoomId, RoomError> {
        let room_id = Uuid::new_v4();
        let _ = sqlx::query("INSERT INTO rooms (id, tenant_id, max_participants, created_at) VALUES ($1, $2, $3, NOW())")
            .bind(room_id)
            .bind(tenant_id)
            .bind(max_participants as i32)
            .execute(&self.db_pool)
            .await;

        let room = Room {
            id: room_id,
            tenant_id,
            peers: HashMap::new(),
            tracks: HashMap::new(),
            max_participants,
            created_at: chrono::Utc::now(),
            state: RoomState::Active,
        };
        self.rooms.insert(room_id, room);
        Ok(RoomId(room_id))
    }

    pub async fn delete_room(&self, room_id: Uuid) -> Result<(), RoomError> {
        let _ = sqlx::query("DELETE FROM rooms WHERE id = $1")
            .bind(room_id)
            .execute(&self.db_pool)
            .await;
        self.rooms.remove(&room_id);
        Ok(())
    }

    pub async fn add_peer(&self, room_id: Uuid, peer_id: Uuid, _transport: Arc<dyn Transport>) -> Result<(), RoomError> {
        if !self.check_room_exists(room_id).await {
            return Err(RoomError::RoomNotFound);
        }
        let peer = Peer {
            id: peer_id,
            connected_at: chrono::Utc::now(),
            tracks_subscribed: std::collections::HashSet::new(),
            bandwidth_estimate: shared::domain::BandwidthEstimate {
                available_upload: 10000,
                available_download: 10000,
                current_usage: 0,
            },
            connection_state: ConnectionState::Connected,
            ice_candidates: Vec::new(),
            dtls_fingerprints: Vec::new(),
        };
        self.peers.insert(peer_id, peer);
        Ok(())
    }

    pub async fn get_room_stats(&self, room_id: Uuid) -> Result<RoomStats, RoomError> {
        let room = self.rooms.get(&room_id).ok_or(RoomError::RoomNotFound)?;
        Ok(RoomStats {
            room_id,
            peer_count: room.peers.len() as u32,
            active_peers: room.peers.values().filter(|p| p.connection_state == ConnectionState::Connected).count() as u32,
            track_count: room.tracks.len() as u32,
            max_participants: room.max_participants,
            created_at: room.created_at,
        })
    }

    pub async fn publish_rtp(&self, _packet: RTPPacket) -> Result<(), PacketError> { Ok(()) }

    pub async fn add_track(&self, room_id: Uuid, track: Track) -> Result<(), RoomError> {
        let mut room = self.rooms.get_mut(&room_id).ok_or(RoomError::RoomNotFound)?;
        room.tracks.insert(track.id, track.clone());
        Ok(())
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
    InternalError,
}

impl std::fmt::Display for RoomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for RoomError {}

impl Clone for SFU {
    fn clone(&self) -> Self {
        Self {
            db_pool: self.db_pool.clone(),
            webrtc_api: self.webrtc_api.clone(),
            rooms: self.rooms.clone(),
            peers: self.peers.clone(),
            relay_tracks: self.relay_tracks.clone(),
            room_sessions: self.room_sessions.clone(),
            sessions: self.sessions.clone(),
            packet_processor: self.packet_processor.clone(),
            logger: self.logger.clone(),
            metrics: self.metrics.clone(),
            error_handler: self.error_handler.clone(),
            worker_handles: self.worker_handles.clone(),
        }
    }
}
