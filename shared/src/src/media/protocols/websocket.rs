use super::*;
use crate::media::{RTPPacket, RTCPPacket};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};
use log::{info, warn, error, debug};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{connect_async, tungstenite::protocol::WebSocketConfig};

pub struct WebSocketAdapter {
    config: ProtocolConfig,
    connections: Arc<RwLock<HashMap<Uuid, WebSocketConnection>>>,
    logger: slog::Logger,
}

#[derive(Debug, Clone)]
struct WebSocketConnection {
    connection_id: Uuid,
    peer_id: Uuid,
    room_id: Uuid,
    ws_stream: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
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

impl WebSocketAdapter {
    pub fn new(logger: slog::Logger) -> Self {
        Self {
            config: ProtocolConfig::default(),
            connections: Arc::new(RwLock::new(HashMap::new())),
            logger,
        }
    }

    async fn handle_websocket_connection(&self, ws_stream: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>) {
        let connection_id = Uuid::new_v4();
        let peer_id = Uuid::new_v4();
        let room_id = Uuid::new_v4();
        
        let mut connection = WebSocketConnection {
            connection_id,
            peer_id,
            room_id,
            ws_stream,
            stats: ConnectionStats {
                connection_id,
                peer_id,
                room_id,
                protocol: ProtocolType::WebSocket,
                start_time: chrono::Utc::now(),
                last_activity: chrono::Utc::now(),
                bytes_sent: 0,
                bytes_received: 0,
                packets_sent: 0,
                packets_received: 0,
                packet_loss: 0.0,
                latency_ms: 0.0,
                jitter_ms: 0.0,
                codec: "VP8".to_string(),
                resolution: Some((1280, 720)),
                framerate: Some(30),
                bitrate_kbps: 2000,
            },
            codec_info: CodecInfo {
                codec: "VP8".to_string(),
                payload_type: 96,
                ssrc: rand::random(),
                clock_rate: 90000,
            },
            buffer: Vec::new(),
        };
        
        self.connections.write().await.insert(connection_id, connection.clone());
        
        // WebSocket processing loop
        loop {
            match connection.ws_stream.next().await {
                Some(Ok(msg)) => {
                    if let Message::Binary(data) = msg {
                        connection.buffer.extend_from_slice(&data);
                        connection.stats.bytes_received += data.len() as u64;
                        connection.stats.last_activity = chrono::Utc::now();
                        
                        self.process_websocket_packet(&mut connection, data).await;
                    }
                }
                Some(Err(e)) => {
                    error!(self.logger, "WebSocket error";
                          "connection_id" => ?connection_id,
                          "error" => %e);
                    break;
                }
                None => {
                    warn!(self.logger, "WebSocket connection closed";
                          "connection_id" => ?connection_id);
                    break;
                }
            }
        }
        
        self.connections.write().await.remove(&connection_id);
    }

    async fn process_websocket_packet(&self, connection: &mut WebSocketConnection, data: Vec<u8>) {
        // WebSocket packets are binary frames
        if data.len() > 1024 {
            let packet_data = data;
            
            let rtp_packet = RTPPacket {
                ssrc: connection.codec_info.ssrc,
                sequence_number: rand::random(),
                timestamp: rand::random(),
                payload_type: connection.codec_info.payload_type,
                payload: packet_data,
                marker: false,
                extension: None,
            };
            
            // self.sfu.handle_packet(rtp_packet, connection.room_id).await;
            connection.stats.packets_received += 1;
        }
    }

    async fn connect_websocket(&self, url: &str) -> Result<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, ProtocolError> {
        let (ws_stream, _) = connect_async(url).await?;
        Ok(ws_stream)
    }
}

impl ProtocolAdapter for WebSocketAdapter {
    fn get_protocol_type(&self) -> ProtocolType {
        ProtocolType::WebSocket
    }

    fn initialize(&mut self, config: ProtocolProtocolConfig) -> Result<(), ProtocolError> {
        info!(self.logger, "Initializing WebSocket adapter";
              "config" => ?config);
        
        self.config = config;
        
        // Start WebSocket server if needed
        // This would typically be handled by a separate service
        
        Ok(())
    }

    fn connect(&mut self, connection_id: Uuid, peer_id: Uuid, room_id: Uuid) -> Result<(), ProtocolError> {
        // WebSocket connections are established via URL
        // This would typically be handled by the client
        Ok(())
    }

    fn disconnect(&mut self, connection_id: Uuid) -> Result<(), ProtocolError> {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(&connection_id) {
            connection.ws_stream.close(None).await?;
        }
        
        Ok(())
    }

    fn send_packet(&mut self, connection_id: Uuid, packet: RTPPacket) -> Result<(), ProtocolError> {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(&connection_id) {
            let result = timeout(Duration::from_millis(100), async {
                let msg = Message::Binary(packet.payload.clone());
                
                if let Err(e) = connection.ws_stream.send(msg).await {
                    error!(self.logger, "WebSocket send error";
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
        // WebSocket doesn't use RTCP directly
        Ok(())
    }

    fn receive_rtcp(&mut self, connection_id: Uuid) -> Result<RTCPPacket, ProtocolError> {
        // WebSocket doesn't use RTCP
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
            protocol: ProtocolType::WebSocket,
            connections: stats,
            aggregate_metrics: ProtocolMetrics {
                protocol: ProtocolType::WebSocket,
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