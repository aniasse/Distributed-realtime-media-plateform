use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub peers: HashMap<Uuid, Peer>,
    pub tracks: HashMap<Uuid, Track>,
    pub max_participants: u32,
    pub created_at: DateTime<Utc>,
    pub state: RoomState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoomState {
    Active,
    Ended,
    Recording,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: Uuid,
    pub connected_at: DateTime<Utc>,
    pub tracks_subscribed: HashSet<Uuid>,
    pub bandwidth_estimate: BandwidthEstimate,
    pub connection_state: ConnectionState,
    pub ice_candidates: Vec<IceCandidate>,
    pub dtls_fingerprints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: Uuid,
    pub publisher_id: Uuid,
    pub kind: MediaKind,
    pub simulcast_layers: Vec<SimulcastLayer>,
    pub ssrc: u32,
    pub payload_type: u8,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaKind {
    Audio,
    Video,
    Data,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulcastLayer {
    pub id: Uuid,
    pub bitrate_kbps: u32,
    pub resolution: Option<(u32, u32)>,
    pub framerate: Option<u32>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthEstimate {
    pub available_upload: u32,
    pub available_download: u32,
    pub current_usage: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionState {
    Connected,
    Disconnected,
    Reconnecting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceCandidate {
    pub foundation: String,
    pub component: u32,
    pub transport: String,
    pub priority: u32,
    pub connection_address: String,
    pub port: u16,
    pub candidate_type: String,
    pub related_address: Option<String>,
    pub related_port: Option<u16>,
    pub username_fragment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherId {
    pub peer_id: Uuid,
    pub track_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionId(pub Uuid);
