use super::*;
use crate::media::{RTPPacket, RTCPPacket};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};
use log::{info, warn, error, debug};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct RTMPAdapter {
    config: ProtocolConfig,
    connections: Arc<RwLock<HashMap<Uuid, RTMPConnection>>>,
    listener: Option<TcpListener>,
    logger: slog::Logger,
}

#[derive(Debug, Clone)]
struct RTMPConnection {
    connection_id: Uuid,
    peer_id: Uuid,
    room_id: Uuid,
    stream_key: String,
    socket: tokio::net::TcpStream,
    stats: ConnectionStats,
    codec_info: CodecInfo,
    buffer: Vec<u8>,
}

#[derive(Debug, Clone)]
struct CodecInfo {
    codec: String,
    payload_type: u8,
    ssrc: u32,
    clock_rate: u32,
}

impl RTMPAdapter {
    pub fn new(logger: slog::Logger) -> Self {
        Self {
            config: ProtocolConfig::default(),
            connections: Arc::new(RwLock::new(HashMap::new())),
            listener: None,
            logger,
        }
    }

    async fn handle_rtmp_connection(&self, stream: tokio::net::TcpStream) {
        // Handle RTMP connection handshake and processing
        let connection_id = Uuid::new_v4();
        let peer_id = Uuid::new_v4();
        let room_id = Uuid::new_v4();
        let stream_key = "default_stream".to_string();
        
        let mut connection = RTMPConnection {
            connection_id,
            peer_id,
            room_id,
            stream_key,
            socket: stream,
            stats: ConnectionStats {
                connection_id,
                peer_id,
                room_id,
                protocol: ProtocolType::RTMP,
                start_time: chrono::Utc::now(),
                last_activity: chrono::Utc::now(),
                bytes_sent: 0,
                bytes_received: 0,
                packets_sent: 0,
                packets_received: 0,
                packet_loss: 0.0,
                latency_ms: 0.0,
                jitter_ms: 0.0,
                codec: "H.264".to_string(),
                resolution: Some((1280, 720)),
                framerate: Some(30),
                bitrate_kbps: 4000,
            },
            codec_info: CodecInfo {
                codec: "H.264".to_string(),
                payload_type: 96,
                ssrc: rand::random(),
                clock_rate: 90000,
            },
            buffer: Vec::new(),
        };
        
        // Add to connections
        self.connections.write().await.insert(connection_id, connection.clone());
        
        // RTMP processing loop
        loop {
            let mut buffer = [0; 4096];
            match connection.socket.read(&mut buffer).await {
                Ok(0) => {
                    warn!(self.logger, "RTMP connection closed";
                          "connection_id" => ?connection_id);
                    break;
                }
                Ok(n) => {
                    connection.buffer.extend_from_slice(&buffer[..n]);
                    connection.stats.bytes_received += n as u64;
                    connection.stats.last_activity = chrono::Utc::now();
                    
                    // Process RTMP packets
                    self.process_rtmp_packet(&mut connection).await;
                }
                Err(e) => {
                    error!(self.logger, "RTMP read error";
                          "connection_id" => ?connection_id,
                          "error" => %e);
                    break;
                }
            }
        }
        
        // Remove connection
        self.connections.write().await.remove(&connection_id);
    }

    async fn process_rtmp_packet(&self, connection: &mut RTMPConnection) {
        // Parse and process RTMP packets
        // This would include handling chunks, messages, and media data
        debug!(self.logger, "Processing RTMP packet";
              "connection_id" => ?connection.connection_id,
              "buffer_size" => connection.buffer.len());
        
        // Simulate RTMP packet processing
        if connection.buffer.len() > 1024 {
            // Process chunk
            let packet_data = connection.buffer.split_off(1024);
            
            // Convert to RTP packet
            let rtp_packet = RTPPacket {
                ssrc: connection.codec_info.ssrc,
                sequence_number: rand::random(),
                timestamp: rand::random(),
                payload_type: connection.codec_info.payload_type,
                payload: packet_data,
                marker: false,
                extension: None,
            };
            
            // Send to SFU
            // self.sfu.handle_packet(rtp_packet, connection.room_id).await;
            
            connection.stats.packets_received += 1;
        }
    }
}

impl ProtocolAdapter for RTMPAdapter {
    fn get_protocol_type(&self) -> ProtocolType {
        ProtocolType::RTMP
    }

    fn initialize(&mut self, config: ProtocolConfig) -> Result<(), ProtocolError> {
        info!(self.logger, "Initializing RTMP adapter";
              "config" => ?config);
        
        self.config = config;
        
        // Start RTMP server
        let listener = TcpListener::bind("0.0.0.0:1935").await?;
        self.listener = Some(listener);
        
        // Start accepting connections
        tokio::spawn(async move {
            while let Ok((stream, _)) = self.listener.as_ref().unwrap().accept().await {
                self.handle_rtmp_connection(stream).await;
            }
        });
        
        Ok(())
    }

    fn connect(&mut self, connection_id: Uuid, peer_id: Uuid, room_id: Uuid) -> Result<(), ProtocolError> {
        // For RTMP, connections are established via TCP, so this is a no-op
        Ok(())
    }

    fn disconnect(&mut self, connection_id: Uuid) -> Result<(), ProtocolError> {
        // Find and close the RTMP connection
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(&connection_id) {
            connection.socket.shutdown().await?;
        }
        
        Ok(())
    }

    fn send_packet(&mut self, connection_id: Uuid, packet: RTPPacket) -> Result<(), ProtocolError> {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(&connection_id) {
            // Convert RTP to RTMP and send
            let rtmp_data = packet.payload;
            
            let result = timeout(Duration::from_millis(100), async {
                if let Err(e) = connection.socket.write_all(&rtmp_data).await {
                    error!(self.logger, "RTMP write error";
                          "connection_id" => ?connection_id,
                          "error" => %e);
                    return Err(ProtocolError::NetworkError);
                }
                
                connection.stats.bytes_sent += rtmp_data.len() as u64;
                connection.stats.packets_sent += 1;
                connection.stats.last_activity = chrono::Utc::now();
                
                Ok(())
            }).await;
            
            match result {
                Ok(Ok(())) => Ok(()),
                Ok(Err(e)) => Err(e),
                Err(_) => Err(ProtocolError::Timeout),
            }
        } else {
            Err(ProtocolError::ConnectionFailed)
        }
    }

    fn receive_packet(&mut self, connection_id: Uuid) -> Result<RTPPacket, ProtocolError> {
        // For RTMP, receiving is handled in the connection loop
        Err(ProtocolError::InvalidPacket)
    }

    fn send_rtcp(&mut self, connection_id: Uuid, packet: RTCPPacket) -> Result<(), ProtocolError> {
        // RTMP doesn't use RTCP, so this is a no-op
        Ok(())
    }

    fn receive_rtcp(&mut self, connection_id: Uuid) -> Result<RTCPPacket, ProtocolError> {
        // RTMP doesn't use RTCP, so this returns an error
        Err(ProtocolError::InvalidPacket)
    }

    fn get_stats(&self, connection_id: Uuid) -> Result<ConnectionStats, ProtocolError> {
        let connections = self.connections.read().await;
        if let Some(connection) = connections.get(&connection_id) {
            Ok(connection.stats.clone())
        } else {
            Err(ProtocolError::ConnectionFailed)
        }
    }

    fn get_aggregate_stats(&self) -> Result<ProtocolStats, ProtocolError> {
        let connections = self.connections.read().await;
        let mut stats = HashMap::new();
        
        for (connection_id, connection) in connections.iter() {
            stats.insert(*connection_id, connection.stats.clone());
        }
        
        Ok(ProtocolStats {
            protocol: ProtocolType::RTMP,
            connections: stats,
            aggregate_metrics: ProtocolMetrics {
                protocol: ProtocolType::RTMP,
                total_connections: connections.len() as u64,
                active_connections: connections.len() as u64,
                total_data_sent: connections.values().map(|c| c.stats.bytes_sent).sum(),
                total_data_received: connections.values().map(|c| c.stats.bytes_received).sum(),
                avg_latency_ms: connections.values().map(|c| c.stats.latency_ms).sum::<f64>() / connections.len() as f64,
                packet_loss_percent: connections.values().map(|c| c.stats.packet_loss).sum::<f64>() / connections.len() as f64,
                bitrate_kbps: connections.values().map(|c| c.stats.bitrate_kbps as f64).sum::<f64>() / connections.len() as f64,
            },
        })
    }
}