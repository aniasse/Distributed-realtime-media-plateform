use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn, error, debug};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolType {
    WebRTC,
    RTMP,
    HLS,
    DASH,
    SRT,
    WebSocket,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    pub protocol: ProtocolType,
    pub enabled: bool,
    pub settings: HashMap<String, String>,
    pub quality_preset: QualityPreset,
    pub network_profile: NetworkProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityPreset {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkProfile {
    Good,
    Average,
    Poor,
    VeryPoor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMetrics {
    pub protocol: ProtocolType,
    pub total_connections: u64,
    pub active_connections: u64,
    pub total_data_sent: u64,
    pub total_data_received: u64,
    pub avg_latency_ms: f64,
    pub packet_loss_percent: f64,
    pub bitrate_kbps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolStats {
    pub protocol: ProtocolType,
    pub connections: HashMap<Uuid, ConnectionStats>,
    pub aggregate_metrics: ProtocolMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStats {
    pub connection_id: Uuid,
    pub peer_id: Uuid,
    pub room_id: Uuid,
    pub protocol: ProtocolType,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub packet_loss: f64,
    pub latency_ms: f64,
    pub jitter_ms: f64,
    pub codec: String,
    pub resolution: Option<(u32, u32)>,
    pub framerate: Option<u32>,
    pub bitrate_kbps: u32,
}

pub trait ProtocolAdapter {
    fn get_protocol_type(&self) -> ProtocolType;
    fn initialize(&mut self, config: ProtocolConfig) -> Result<(), ProtocolError>;
    fn connect(&mut self, connection_id: Uuid, peer_id: Uuid, room_id: Uuid) -> Result<(), ProtocolError>;
    fn disconnect(&mut self, connection_id: Uuid) -> Result<(), ProtocolError>;
    fn send_packet(&mut self, connection_id: Uuid, packet: crate::media::RTPPacket) -> Result<(), ProtocolError>;
    fn receive_packet(&mut self, connection_id: Uuid) -> Result<crate::media::RTPPacket, ProtocolError>;
    fn send_rtcp(&mut self, connection_id: Uuid, packet: crate::media::RTCPPacket) -> Result<(), ProtocolError>;
    fn receive_rtcp(&mut self, connection_id: Uuid) -> Result<crate::media::RTCPPacket, ProtocolError>;
    fn get_stats(&self, connection_id: Uuid) -> Result<ConnectionStats, ProtocolError>;
    fn get_aggregate_stats(&self) -> Result<ProtocolStats, ProtocolError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolError {
    ConnectionFailed,
    AuthenticationFailed,
    InvalidPacket,
    UnsupportedCodec,
    NetworkError,
    Timeout,
    ConfigurationError,
    InternalError,
}

pub struct ProtocolManager {
    protocols: Arc<RwLock<HashMap<ProtocolType, Box<dyn ProtocolAdapter>>>>,
    metrics: Arc<RwLock<HashMap<ProtocolType, ProtocolMetrics>>>,
    logger: slog::Logger,
}

impl ProtocolManager {
    pub fn new(logger: slog::Logger) -> Self {
        Self {
            protocols: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(HashMap::new())),
            logger,
        }
    }

    pub async fn register_protocol(&self, protocol_type: ProtocolType, adapter: Box<dyn ProtocolAdapter>) -> Result<(), ProtocolError> {
        info!(self.logger, "Registering protocol";
              "protocol" => ?protocol_type);
        
        self.protocols.write().await.insert(protocol_type.clone(), adapter);
        self.metrics.write().await.insert(protocol_type.clone(), ProtocolMetrics {
            protocol: protocol_type.clone(),
            total_connections: 0,
            active_connections: 0,
            total_data_sent: 0,
            total_data_received: 0,
            avg_latency_ms: 0.0,
            packet_loss_percent: 0.0,
            bitrate_kbps: 0.0,
        });
        
        Ok(())
    }

    pub async fn get_protocol(&self, protocol_type: &ProtocolType) -> Option<Box<dyn ProtocolAdapter>> {
        self.protocols.read().await.get(protocol_type).cloned()
    }

    pub async fn initialize_protocol(&self, protocol_type: ProtocolType, config: ProtocolConfig) -> Result<(), ProtocolError> {
        if let Some(protocol) = self.protocols.write().await.get_mut(&protocol_type) {
            protocol.initialize(config)?;
            Ok(())
        } else {
            Err(ProtocolError::ConfigurationError)
        }
    }

    pub async fn connect(&self, protocol_type: ProtocolType, connection_id: Uuid, peer_id: Uuid, room_id: Uuid) -> Result<(), ProtocolError> {
        if let Some(protocol) = self.protocols.write().await.get_mut(&protocol_type) {
            protocol.connect(connection_id, peer_id, room_id)?;
            
            let mut metrics = self.metrics.write().await;
            if let Some(protocol_metrics) = metrics.get_mut(&protocol_type) {
                protocol_metrics.total_connections += 1;
                protocol_metrics.active_connections += 1;
            }
            
            Ok(())
        } else {
            Err(ProtocolError::ConnectionFailed)
        }
    }

    pub async fn disconnect(&self, protocol_type: ProtocolType, connection_id: Uuid) -> Result<(), ProtocolError> {
        if let Some(protocol) = self.protocols.write().await.get_mut(&protocol_type) {
            protocol.disconnect(connection_id)?;
            
            let mut metrics = self.metrics.write().await;
            if let Some(protocol_metrics) = metrics.get_mut(&protocol_type) {
                protocol_metrics.active_connections = protocol_metrics.active_connections.saturating_sub(1);
            }
            
            Ok(())
        } else {
            Err(ProtocolError::ConnectionFailed)
        }
    }

    pub async fn send_packet(&self, protocol_type: ProtocolType, connection_id: Uuid, packet: crate::media::RTPPacket) -> Result<(), ProtocolError> {
        if let Some(protocol) = self.protocols.write().await.get_mut(&protocol_type) {
            protocol.send_packet(connection_id, packet)?;
            
            let mut metrics = self.metrics.write().await;
            if let Some(protocol_metrics) = metrics.get_mut(&protocol_type) {
                protocol_metrics.total_data_sent += packet.payload.len() as u64;
            }
            
            Ok(())
        } else {
            Err(ProtocolError::InvalidPacket)
        }
    }

    pub async fn receive_packet(&self, protocol_type: ProtocolType, connection_id: Uuid) -> Result<crate::media::RTPPacket, ProtocolError> {
        if let Some(protocol) = self.protocols.write().await.get_mut(&protocol_type) {
            let packet = protocol.receive_packet(connection_id)?;
            
            let mut metrics = self.metrics.write().await;
            if let Some(protocol_metrics) = metrics.get_mut(&protocol_type) {
                protocol_metrics.total_data_received += packet.payload.len() as u64;
            }
            
            Ok(packet)
        } else {
            Err(ProtocolError::InvalidPacket)
        }
    }

    pub async fn get_stats(&self, protocol_type: ProtocolType, connection_id: Uuid) -> Result<ConnectionStats, ProtocolError> {
        if let Some(protocol) = self.protocols.read().await.get(&protocol_type) {
            protocol.get_stats(connection_id)
        } else {
            Err(ProtocolError::ConnectionFailed)
        }
    }

    pub async fn get_aggregate_stats(&self, protocol_type: ProtocolType) -> Result<ProtocolStats, ProtocolError> {
        if let Some(protocol) = self.protocols.read().await.get(&protocol_type) {
            let connections: HashMap<Uuid, ConnectionStats> = protocol.get_aggregate_stats()?.connections;
            let aggregate_metrics = self.metrics.read().await.get(&protocol_type).cloned().unwrap_or_default();
            
            Ok(ProtocolStats {
                protocol: protocol_type,
                connections,
                aggregate_metrics,
            })
        } else {
            Err(ProtocolError::ConnectionFailed)
        }
    }

    pub async fn list_protocols(&self) -> Vec<ProtocolType> {
        self.protocols.read().await.keys().cloned().collect()
    }
}