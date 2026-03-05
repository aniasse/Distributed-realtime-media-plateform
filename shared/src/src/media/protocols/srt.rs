use super::*;
use crate::media::{RTPPacket, RTCPPacket};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};
use log::{info, warn, error, debug};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct SRTAdapter {
    config: ProtocolConfig,
    connections: Arc<RwLock<HashMap<Uuid, SRTConnection>>>,
    listener: Option<TcpListener>,
    logger: slog::Logger,
}

#[derive(Debug, Clone)]
struct SRTConnection {
    connection_id: Uuid,
    peer_id: Uuid,
    room_id: Uuid,
    socket: tokio::net::TcpStream,
    stats: ConnectionStats,
    codec_info: CodecInfo,
    buffer: Vec<u8>,
    latency: u32,
    packet_loss: f64,
}

#[derive(Debug, Clone)]
struct CodecInfo {
    codec: String,
    payload_type: u8,
    ssrc: u32,
    clock_rate: u32,
}

impl SRTAdapter {
    pub fn new(logger: slog::Logger) -> Self {
        Self {
            config: ProtocolConfig::default(),
            connections: Arc::new(RwLock::new(HashMap::new())),
            listener: None,
            logger,
        }
    }

    async fn handle_srt_connection(&self, stream: tokio::net::TcpStream) {
        let connection_id = Uuid::new_v4();
        let peer_id = Uuid::new_v4();
        let room_id = Uuid::new_v4();
        
        let mut connection = SRTConnection {
            connection_id,
            peer_id,
            room_id,
            socket: stream,
            stats: ConnectionStats {
                connection_id,
                peer_id,
                room_id,
                protocol: ProtocolType::SRT,
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
                bitrate_kbps: 6000,
            },
            codec_info: CodecInfo {
                codec: "H.264".to_string(),
                payload_type: 96,
                ssrc: rand::random(),
                clock_rate: 90000,
            },
            buffer: Vec::new(),
            latency: 120, // Default SRT latency
            packet_loss: 0.0,
        };
        
        self.connections.write().await.insert(connection_id, connection.clone());
        
        loop {
            let mut buffer = [0; 4096];
            match connection.socket.read(&mut buffer).await {
                Ok(0) => {
                    warn!(self.logger, "SRT connection closed";
                          "connection_id" => ?connection_id);
                    break;
                }
                Ok(n) => {
                    connection.buffer.extend_from_slice(&buffer[..n]);
                    connection.stats.bytes_received += n as u64;
                    connection.stats.last_activity = chrono::Utc::now();
                    
                    self.process_srt_packet(&mut connection).await;
                }
                Err(e) => {
                    error!(self.logger, "SRT read error";
                          "connection_id" => ?connection_id,
                          "error" => %e);
                    break;
                }
            }
        }
        
        self.connections.write().await.remove(&connection_id);
    }

    async fn process_srt_packet(&self, connection: &mut SRTConnection) {
        if connection.buffer.len() > 1024 {
            let packet_data = connection.buffer.split_off(1024);
            
            let rtp_packet = RTPPacket {
                ssrc: connection.codec_info.ssrc,
                sequence_number: rand::random(),
                timestamp: rand::random(),
                payload_type: connection.codec_info.payload_type,
                payload: packet_data,
                marker: false,
                extension: None,
            };
            
            // Simulate SRT packet loss and latency
            if rand::random::<f64>() > connection.packet_loss {
                // self.sfu.handle_packet(rtp_packet, connection.room_id).await;
                connection.stats.packets_received += 1;
            } else {
                debug!(self.logger, "SRT packet loss";
                      "connection_id" => ?connection.connection_id);
            }
        }
    }
}

impl ProtocolAdapter for SRTAdapter {
    fn get_protocol_type(&self) -> ProtocolType {
        ProtocolType::SRT
    }

    fn initialize(&mut self, config: ProtocolConfig) -> Result<(), ProtocolError> {
        info!(self.logger, "Initializing SRT adapter";
              "config" => ?config);
        
        self.config = config;
        
        // Start SRT server
        let listener = TcpListener::bind("0.0.0.0:5000").await?;
        self.listener = Some(listener);
        
        tokio::spawn(async move {
            while let Ok((stream, _)) = self.listener.as_ref().unwrap().accept().await {
                self.handle_srt_connection(stream).await;
            }
        });
        
        Ok(())
    }

    fn connect(&mut self, connection_id: Uuid, peer_id: Uuid, room_id: Uuid) -> Result<(), ProtocolError> {
        // SRT connections are established via TCP
        Ok(())
    }

    fn disconnect(&mut self, connection_id: Uuid) -> Result<(), ProtocolError> {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(&connection_id) {
            connection.socket.shutdown().await?;
        }
        
        Ok(())
    }

    fn send_packet(&mut self, connection_id: Uuid, packet: RTPPacket) -> Result<(), ProtocolError> {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(&connection_id) {
            let result = timeout(Duration::from_millis(100), async {
                if let Err(e) = connection.socket.write_all(&packet.payload).await {
                    error!(self.logger, "SRT write error";
                          "connection_id" => ?connection_id,
                          "error" => %e);
                    return Err(ProtocolError::NetworkError);
                }
                
                connection.stats.bytes_sent += packet.payload.len() as u64;
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
        Err(ProtocolError::InvalidPacket)
    }

    fn send_rtcp(&mut self, connection_id: Uuid, packet: RTCPPacket) -> Result<(), ProtocolError> {
        // SRT doesn't use RTCP
        Ok(())
    }

    fn receive_rtcp(&mut self, connection_id: Uuid) -> Result<RTCPPacket, ProtocolError> {
        // SRT doesn't use RTCP
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
            protocol: ProtocolType::SRT,
            connections: stats,
            aggregate_metrics: ProtocolMetrics {
                protocol: ProtocolType::SRT,
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