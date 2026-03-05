use super::*;
use crate::media::{RTPPacket, RTCPPacket};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};
use log::{info, warn, error, debug};
use dash::{self, common::uri::Uri};

pub struct DASHAdapter {
    config: ProtocolConfig,
    segments: Arc<RwLock<HashMap<Uuid, DASHSegment>>>,
    manifest_manager: dash::ManifestManager,
    logger: slog::Logger,
}

#[derive(Debug, Clone)]
struct DASHSegment {
    id: Uuid,
    representation_id: String,
    url: String,
    duration: f64,
    sequence_number: u32,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl DASHAdapter {
    pub fn new(logger: slog::Logger) -> Self {
        Self {
            config: ProtocolConfig::default(),
            segments: Arc::new(RwLock::new(HashMap::new())),
            manifest_manager: dash::ManifestManager::new(),
            logger,
        }
    }

    async fn add_segment_to_manifest(&self, segment: &DASHSegment) {
        let representation = self.manifest_manager.get_representation("video", "1");
        
        let segment_info = dash::SegmentInfo {
            url: Uri::new(segment.url.clone()),
            duration: segment.duration,
            sequence_number: segment.sequence_number,
        };
        
        representation.add_segment(segment_info);
        
        debug!(self.logger, "Added segment to DASH manifest";
              "segment_id" => ?segment.id,
              "representation" => "video/1");
    }
}

impl ProtocolAdapter for DASHAdapter {
    fn get_protocol_type(&self) -> ProtocolType {
        ProtocolType::DASH
    }

    fn initialize(&mut self, config: ProtocolConfig) -> Result<(), ProtocolError> {
        info!(self.logger, "Initializing DASH adapter";
              "config" => ?config);
        
        self.config = config;
        
        // Initialize DASH manifest
        self.manifest_manager.initialize("output/dash").await?;
        
        Ok(())
    }

    fn connect(&mut self, connection_id: Uuid, peer_id: Uuid, room_id: Uuid) -> Result<(), ProtocolError> {
        // DASH doesn't have traditional connections
        Ok(())
    }

    fn disconnect(&mut self, connection_id: Uuid) -> Result<(), ProtocolError> {
        // DASH doesn't have traditional connections
        Ok(())
    }

    fn send_packet(&mut self, connection_id: Uuid, packet: RTPPacket) -> Result<(), ProtocolError> {
        let segment_id = Uuid::new_v4();
        let representation_id = "video/1".to_string();
        let url = format!("/dash/{}.m4s", segment_id);
        
        let segment = DASHSegment {
            id: segment_id,
            representation_id: representation_id.clone(),
            url,
            duration: 2.0, // 2-second segments
            sequence_number: 1, // This would increment
            created_at: chrono::Utc::now(),
        };
        
        // Store segment
        self.segments.write().await.insert(segment_id, segment.clone());
        
        // Add to manifest
        self.add_segment_to_manifest(&segment).await;
        
        Ok(())
    }

    fn receive_packet(&mut self, connection_id: Uuid) -> Result<RTPPacket, ProtocolError> {
        // DASH is output-only
        Err(ProtocolError::InvalidPacket)
    }

    fn send_rtcp(&mut self, connection_id: Uuid, packet: RTCPPacket) -> Result<(), ProtocolError> {
        // DASH doesn't use RTCP
        Ok(())
    }

    fn receive_rtcp(&mut self, connection_id: Uuid) -> Result<RTCPPacket, ProtocolError> {
        // DASH doesn't use RTCP
        Err(ProtocolError::InvalidPacket)
    }

    fn get_stats(&self, connection_id: Uuid) -> Result<ConnectionStats, ProtocolError> {
        // DASH doesn't have traditional connections
        Err(ProtocolError::ConnectionFailed)
    }

    fn get_aggregate_stats(&self) -> Result<ProtocolStats, ProtocolError> {
        let segments = self.segments.read().await;
        let total_segments = segments.len() as u64;
        
        Ok(ProtocolStats {
            protocol: ProtocolType::DASH,
            connections: HashMap::new(),
            aggregate_metrics: ProtocolMetrics {
                protocol: ProtocolType::DASH,
                total_connections: total_segments,
                active_connections: total_segments,
                total_data_sent: 0, // Segments are stored separately
                total_data_received: 0,
                avg_latency_ms: 0.0,
                packet_loss_percent: 0.0,
                bitrate_kbps: 0.0,
            },
        })
    }
}