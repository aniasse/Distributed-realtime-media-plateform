use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::fs;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use log::{info, error, debug};

use crate::shared::domain::{Room, Track, RoomId, TrackId};
use crate::shared::media::{RTPPacket, RTCPPacket, StorageBackend};
use crate::shared::utils::{Logger, Metrics, ErrorHandler};

pub struct RecordingService {
    pub storage: Box<dyn StorageBackend>,
    pub rooms: Arc<Mutex<HashMap<Uuid, Room>>>,
    pub tracks: Arc<Mutex<HashMap<Uuid, Track>>>,
    pub logger: Logger,
    pub metrics: Metrics,
    pub error_handler: ErrorHandler,
}

impl RecordingService {
    pub fn new(storage: Box<dyn StorageBackend>) -> Self {
        Self {
            storage,
            rooms: Arc::new(Mutex::new(HashMap::new())),
            tracks: Arc::new(Mutex::new(HashMap::new())),
            logger: Logger::new("recording"),
            metrics: Metrics::new(),
            error_handler: ErrorHandler::new("recording"),
        }
    }

    pub async fn start(&self) -> Result<(), std::io::Error> {
        self.logger.info("Starting Recording service");
        
        // Initialize storage
        self.storage.initialize().await?;
        
        Ok(())
    }

    pub async fn record_room(&self, room_id: Uuid) -> Result<(), RecordingError> {
        self.logger.info(&format!("Starting recording for room {}", room_id));
        
        // Create room directory
        let room_dir = format!("/recordings/{}/", room_id);
        fs::create_dir_all(&room_dir).await.map_err(|e| RecordingError::StorageError(e.into()))?;
        
        // Add room to active recordings
        let mut rooms = self.rooms.lock().unwrap();
        rooms.insert(room_id, Room {
            id: room_id,
            tenant_id: Uuid::new_v4(), // Would come from actual tenant system
            peers: HashMap::new(),
            tracks: HashMap::new(),
            max_participants: 100,
            created_at: chrono::Utc::now(),
            state: crate::shared::domain::RoomState::Recording,
        });
        
        self.metrics.increment_counter("recordings_started", 1);
        
        Ok(())
    }

    pub async fn stop_recording(&self, room_id: Uuid) -> Result<(), RecordingError> {
        self.logger.info(&format!("Stopping recording for room {}", room_id));
        
        // Remove from active recordings
        let mut rooms = self.rooms.lock().unwrap();
        rooms.remove(&room_id);
        
        self.metrics.increment_counter("recordings_stopped", 1);
        
        Ok(())
    }

    pub async fn record_packet(&self, packet: RTPPacket, room_id: Uuid) -> Result<(), RecordingError> {
        self.logger.debug(&format!("Recording packet for room {}", room_id));
        
        // Find the track
        let tracks = self.tracks.lock().unwrap();
        if let Some(track) = tracks.values().find(|t| t.ssrc == packet.ssrc) {
            // Create segment file
            let segment_file = format!("/recordings/{}/{}_{}.rtp", room_id, track.id, packet.sequence_number);
            
            // Write packet to file
            let data = bincode::serialize(&packet).map_err(|e| RecordingError::SerializationError(e))?;
            
            self.storage.save_segment(room_id, data).await.map_err(|e| RecordingError::StorageError(e))?;
            
            self.metrics.increment_counter("packets_recorded", 1);
            Ok(())
        } else {
            Err(RecordingError::TrackNotFound)
        }
    }

    pub async fn record_rtcp(&self, packet: RTCPPacket, room_id: Uuid) -> Result<(), RecordingError> {
        self.logger.debug(&format!("Recording RTCP packet for room {}", room_id));
        
        // Create RTCP segment file
        let segment_file = format!("/recordings/{}/rtcp_{}.rtcp", room_id, packet.sender_ssrc);
        
        // Write RTCP packet to file
        let data = bincode::serialize(&packet).map_err(|e| RecordingError::SerializationError(e))?;
        
        self.storage.save_segment(room_id, data).await.map_err(|e| RecordingError::StorageError(e))?;
        
        self.metrics.increment_counter("rtcp_packets_recorded", 1);
        Ok(())
    }

    pub async fn get_recording_info(&self, room_id: Uuid) -> Result<RecordingInfo, RecordingError> {
        self.logger.info(&format!("Getting recording info for room {}", room_id));
        
        let rooms = self.rooms.lock().unwrap();
        if let Some(room) = rooms.get(&room_id) {
            let tracks = self.tracks.lock().unwrap();
            let track_count = tracks.len();
            let duration = chrono::Utc::now() - room.created_at;
            
            Ok(RecordingInfo {
                room_id,
                track_count: track_count as u32,
                duration_seconds: duration.num_seconds() as u32,
                status: room.state.clone(),
            })
        } else {
            Err(RecordingError::RoomNotFound)
        }
    }

    pub async fn list_recordings(&self) -> Result<Vec<RecordingInfo>, RecordingError> {
        self.logger.info("Listing all recordings");
        
        let rooms = self.rooms.lock().unwrap();
        let mut recordings = Vec::new();
        
        for (room_id, room) in rooms.iter() {
            let tracks = self.tracks.lock().unwrap();
            let track_count = tracks.len();
            let duration = chrono::Utc::now() - room.created_at;
            
            recordings.push(RecordingInfo {
                room_id: *room_id,
                track_count: track_count as u32,
                duration_seconds: duration.num_seconds() as u32,
                status: room.state.clone(),
            });
        }
        
        Ok(recordings)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RecordingInfo {
    pub room_id: Uuid,
    pub track_count: u32,
    pub duration_seconds: u32,
    pub status: crate::shared::domain::RoomState,
}

#[derive(Debug)]
pub enum RecordingError {
    StorageError(std::io::Error),
    SerializationError(bincode::Error),
    RoomNotFound,
    TrackNotFound,
    InternalError,
}

impl std::fmt::Display for RecordingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for RecordingError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_recording_service_creation() {
        let storage = MockStorage::new();
        let recording_service = RecordingService::new(storage);
        
        assert!(recording_service.rooms.lock().unwrap().is_empty());
        assert!(recording_service.tracks.lock().unwrap().is_empty());
    }
    
    #[tokio::test]
    async fn test_record_room() {
        let storage = MockStorage::new();
        let recording_service = RecordingService::new(storage);
        
        let room_id = Uuid::new_v4();
        recording_service.record_room(room_id).await.unwrap();
        
        assert!(recording_service.rooms.lock().unwrap().contains_key(&room_id));
        assert_eq!(recording_service.metrics.get_counter("recordings_started").unwrap(), 1);
    }
    
    struct MockStorage;
    impl MockStorage {
        fn new() -> Self { Self }
    }
    
    impl StorageBackend for MockStorage {
        async fn initialize(&self) -> Result<(), std::io::Error> {
            Ok(())
        }
        async fn save_segment(&self, _room_id: Uuid, _data: Vec<u8>) -> Result<(), std::io::Error> {
            Ok(())
        }
        async fn get_segment(&self, _room_id: Uuid, _segment_id: &str) -> Result<Vec<u8>, std::io::Error> {
            unimplemented!()
        }
        async fn list_segments(&self, _room_id: Uuid) -> Result<Vec<String>, std::io::Error> {
            unimplemented!()
        }
    }
}