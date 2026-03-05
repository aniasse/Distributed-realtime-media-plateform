use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RTPPacket {
    pub ssrc: u32,
    pub sequence_number: u16,
    pub timestamp: u32,
    pub payload_type: u8,
    pub payload: Vec<u8>,
    pub marker: bool,
    pub extension: Option<RTPExtension>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RTPExtension {
    pub id: u8,
    pub length: u16,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RTCPPacket {
    pub packet_type: RTCPPacketType,
    pub sender_ssrc: u32,
    pub media_ssrc: u32,
    pub report_count: u8,
    pub payload: Vec<u8>,
    pub timestamp: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RTCPPacketType {
    SenderReport,
    ReceiverReport,
    SDES,
    BYE,
    APP,
    RTPFB,
    PSFB,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NACK {
    pub pid: u16,
    pub blp: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PLI {
    pub ssrc: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FIR {
    pub ssrc: u32,
    pub command_seq: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SLI {
    pub first: u16,
    pub number: u16,
    pub picture_id: u16,
}

pub trait Transport {
    fn send_packet(&self, packet: RTPPacket) -> Result<(), TransportError>;
    fn receive_packet(&self) -> Result<RTPPacket, TransportError>;
    fn send_rtcp(&self, packet: RTCPPacket) -> Result<(), TransportError>;
    fn receive_rtcp(&self) -> Result<RTCPPacket, TransportError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportError {
    ConnectionFailed,
    SendFailed,
    ReceiveFailed,
    AuthenticationFailed,
    Timeout,
}

pub trait PacketProcessor {
    fn process_rtp(&self, packet: RTPPacket) -> Result<(), PacketError>;
    fn process_rtcp(&self, packet: RTCPPacket) -> Result<(), PacketError>;
    fn get_forwarding_strategy(&self, track_id: Uuid) -> ForwardingStrategy;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ForwardingStrategy {
    Unicast { peer_ids: Vec<Uuid> },
    Multicast { group_id: Uuid },
    Simulcast { layers: Vec<Uuid> },
    SVC { layers: Vec<Uuid> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PacketError {
    InvalidPacket,
    UnsupportedCodec,
    EncryptionFailed,
    DecryptionFailed,
    ProcessingTimeout,
}
