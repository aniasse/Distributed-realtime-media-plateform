use super::*;
use crate::media::{RTPPacket, RTCPPacket};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};
use log::{info, warn, error, debug};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub struct HLSAdapter {
    config: ProtocolConfig,
    segments: Arc<RwLock<HashMap<Uuid, HLSSegment>>>,
    output_dir: String,
    logger: slog::Logger,
}

#[derive(Debug, Clone)]
struct HLSSegment {
    id: Uuid,
    room_id: Uuid,
    sequence_number: u32,
    duration: f64,
    file_path: String,
    data: Vec<u8>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl HLSAdapter {
    pub fn new(logger: slog::Logger) -> Self {
        Self {
            config: ProtocolConfig::default(),
            segments: Arc::new(RwLock::new(HashMap::new())),
            output_dir: "./hls_output".to_string(),
            logger,
        }
    }

    async fn write_segment(&self, segment: &HLSSegment) -> Result<(), ProtocolError> {
        let path = Path::new(&self.output_dir);
        if !path.exists() {
            tokio::fs::create_dir_all(path).await?;
        }
        
        let file_path = format!("{}/{}.ts", self.output_dir, segment.id);
        let mut file = File::create(&file_path).await?;
        let mut writer = BufWriter::new(file);
        
        writer.write_all(&segment.data).await?;
        
        debug!(self.logger, "Wrote HLS segment";
              "segment_id" => ?segment.id,
              "file_path" => file_path);
        
        Ok(())
    }
}

impl ProtocolAdapter for HLSAdapter {
    fn get_protocol_type(&self) -> ProtocolType {
        ProtocolType::HLS
    }

    fn initialize(&mut self, config: ProtocolConfig) -> Result<(), ProtocolError> {
        info!(self.logger, "Initializing HLS adapter";
              "config" => ?config);
        
        self.config = config;
        
        // Create output directory
        let path = Path::new(&self.output_dir);
        if !path.exists() {
            tokio::fs::create_dir_all(path).await?;
        }
        
        Ok(())
    }

    fn connect(&mut self, connection_id: Uuid, peer_id: Uuid, room_id: Uuid) -> Result<(), ProtocolError> {
        // HLS doesn't have traditional connections, so this is a no-op
        Ok(())
    }

    fn disconnect(&mut self, connection_id: Uuid) -> Result<(), ProtocolError> {
        // HLS doesn't have traditional connections, so this is a no-op
        Ok(())
    }

    fn send_packet(&mut self, connection_id: Uuid, packet: RTPPacket) -> Result<(), ProtocolError> {
        // HLS works by creating segments from RTP packets
        let segment_id = Uuid::new_v4();
        let room_id = Uuid::new_v4(); // This would come from context
        
        let segment = HLSSegment {
            id: segment_id,
            room_id,
            sequence_number: 1, // This would increment
            duration: 6.0, // 6-second segments
            file_path: "".to_string(),
            data: packet.payload.clone(),
            created_at: chrono::Utc::now(),
        };
        
        // Store segment
        self.segments.write().await.insert(segment_id, segment.clone());
        
        // Write to file
        self.write_segment(&segment).await?;
        
        Ok(())
    }

    fn receive_packet(&mut self, connection_id: Uuid) -> Result<RTPPacket, ProtocolError> {
        // HLS is output-only, so receiving packets doesn't make sense
        Err(ProtocolError::InvalidPacket)
    }

    fn send_rtcp(&mut self, connection_id: Uuid, packet: RTCPPacket) -> Result<(), ProtocolError> {
        // HLS doesn't use RTCP
        Ok(())
    }

    fn receive_rtcp(&mut self, connection_id: Uuid) -> Result<RTCPPacket, ProtocolError> {
        // HLS doesn't use RTCP
        Err(ProtocolError::InvalidPacket)
    }

    fn get_stats(&self, connection_id: Uuid) -> Result<ConnectionStats, ProtocolError> {
        // HLS doesn't have traditional connections
        Err(ProtocolError::ConnectionFailed)
    }

    fn get_aggregate_stats(&self) -> Result<ProtocolStats, ProtocolError> {
        let segments = self.segments.read().await;
        let total_segments = segments.len() as u64;
        
        Ok(ProtocolStats {
            protocol: ProtocolType::HLS,
            connections: HashMap::new(),
            aggregate_metrics: ProtocolMetrics {
                protocol: ProtocolType::HLS,
                total_connections: total_segments,
                active_connections: total_segments,
                total_data_sent: segments.values().map(|s| s.data.len() as u64).sum(),
                total_data_received: 0,
                avg_latency_ms: 0.0,
                packet_loss_percent: 0.0,
                bitrate_kbps: 0.0,
            },
        })
    }
}