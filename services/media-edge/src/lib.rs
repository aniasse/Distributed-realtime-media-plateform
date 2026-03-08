use std::collections::HashMap;
use std::sync::{Arc};
use tokio::sync::{Mutex};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use log::{info};
use chrono::{DateTime, Utc};

use shared::domain::{Room, Peer};
use shared::media::{Transport};
use shared::security::{AuthProvider};
use shared::utils::{Logger, Metrics, ErrorHandler};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StreamStatus {
    Active,
    Idle,
    Error,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StreamInfo {
    pub stream_key: String,
    pub stream_type: String,
    pub status: StreamStatus,
    pub connected_peers: u32,
    pub bitrate: u32,
    pub resolution: (u32, u32),
    pub created_at: DateTime<Utc>,
}

pub struct MediaEdge {
    pub transport: Box<dyn Transport + Send + Sync>,
    pub auth_provider: Box<dyn AuthProvider>,
    pub rooms: Arc<Mutex<HashMap<Uuid, Room>>>,
    pub peers: Arc<Mutex<HashMap<Uuid, Peer>>>,
    pub active_streams: Arc<Mutex<HashMap<String, StreamInfo>>>,
    pub logger: Logger,
    pub metrics: Arc<Mutex<Metrics>>,
    pub error_handler: ErrorHandler,
}

impl MediaEdge {
    pub fn new(
        transport: Box<dyn Transport + Send + Sync>,
        auth_provider: Box<dyn AuthProvider>,
    ) -> Self {
        Self {
            transport,
            auth_provider,
            rooms: Arc::new(Mutex::new(HashMap::new())),
            peers: Arc::new(Mutex::new(HashMap::new())),
            active_streams: Arc::new(Mutex::new(HashMap::new())),
            logger: Logger::new("media-edge"),
            metrics: Arc::new(Mutex::new(Metrics::new())),
            error_handler: ErrorHandler::new("media-edge"),
        }
    }

    pub async fn start_stream(
        &self,
        stream_key: &str,
        stream_type: &str,
        resolution: Option<(u32, u32)>,
        bitrate: Option<u32>
    ) -> Result<(), MediaError> {
        self.logger.info(&format!("Starting stream {}", stream_key));
        
        if !self.auth_provider.validate_stream_key(stream_key) {
            return Err(MediaError::InvalidStreamKey);
        }
        
        let mut streams = self.active_streams.lock().unwrap();
        streams.insert(stream_key.to_string(), StreamInfo {
            stream_key: stream_key.to_string(),
            stream_type: stream_type.to_string(),
            status: StreamStatus::Active,
            connected_peers: 0,
            bitrate: bitrate.unwrap_or(2500),
            resolution: resolution.unwrap_or((1920, 1080)),
            created_at: Utc::now(),
        });
        
        Ok(())
    }

    pub async fn stop_stream(&self, stream_key: &str) -> Result<(), MediaError> {
        self.logger.info(&format!("Stopping stream {}", stream_key));
        
        let mut streams = self.active_streams.lock().unwrap();
        if streams.remove(stream_key).is_some() {
            Ok(())
        } else {
            Err(MediaError::StreamNotFound)
        }
    }

    pub async fn list_streams(&self) -> Result<Vec<StreamInfo>, MediaError> {
        let streams = self.active_streams.lock().unwrap();
        Ok(streams.values().cloned().collect())
    }
}

#[derive(Debug)]
pub enum MediaError {
    InvalidStreamKey,
    StreamNotFound,
    InternalError,
}

impl std::fmt::Display for MediaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for MediaError {}

impl Clone for MediaEdge {
    fn clone(&self) -> Self {
        // Same clone issue as RecordingService, using Arc in main.rs instead
        panic!("MediaEdge clone not implemented");
    }
}
