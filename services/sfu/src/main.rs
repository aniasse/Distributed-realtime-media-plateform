use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::{Serialize, Deserialize};
use serde_json::json;
use log::{info, error};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use sqlx::PgPool;

// Use the library crate name instead of mod lib
use sfu::{SFU};

#[derive(Serialize, Deserialize)]
pub struct CreateRoomRequest {
    pub tenant_id: Uuid,
    pub max_participants: u32,
}

#[derive(Serialize, Deserialize)]
pub struct RoomResponse {
    pub room_id: Uuid,
    pub tenant_id: Uuid,
    pub max_participants: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct AddPeerRequest {
    pub room_id: Uuid,
    pub peer_id: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct AddTrackRequest {
    pub room_id: Uuid,
    pub track: Track,
}

#[derive(Serialize, Deserialize)]
pub struct Track {
    pub id: Uuid,
    pub publisher_id: Uuid,
    pub kind: String,
    pub ssrc: u32,
    pub media_info: MediaInfo,
}

#[derive(Serialize, Deserialize)]
pub struct MediaInfo {
    pub codec: String,
    pub bitrate: u32,
    pub resolution: Option<(String, String)>,
    pub framerate: Option<u32>,
}

#[derive(Serialize, Deserialize)]
pub struct RTPPacket {
    pub ssrc: u32,
    pub payload: Vec<u8>,
    pub timestamp: u32,
    pub sequence_number: u16,
    pub marker: bool,
}

#[derive(Serialize, Deserialize)]
pub struct RoomStats {
    pub room_id: Uuid,
    pub peer_count: u32,
    pub active_peers: u32,
    pub track_count: u32,
    pub max_participants: u32,
    pub created_at: DateTime<Utc>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:password@localhost/drmp".to_string());
    let db_pool = PgPool::connect(&database_url).await.expect("Failed to connect to database");
    
    let packet_processor = MockPacketProcessor::new();
    let sfu = SFU::new(db_pool, Box::new(packet_processor));
    
    info!("Starting SFU service on port 5004");
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(sfu.clone()))
            .service(
                web::resource("/api/rooms")
                    .route(web::post().to(create_room))
            )
            .service(
                web::resource("/api/rooms/{room_id}")
                    .route(web::delete().to(delete_room))
            )
            .service(
                web::resource("/api/peers")
                    .route(web::post().to(add_peer))
            )
            .service(
                web::resource("/api/tracks")
                    .route(web::post().to(add_track))
            )
            .service(
                web::resource("/api/rooms/{room_id}/stats")
                    .route(web::get().to(get_room_stats))
            )
            .service(
                web::resource("/api/publish")
                    .route(web::post().to(publish_packet))
            )
            .service(
                web::resource("/ws")
                    .route(web::get().to(sfu::signaling::ws_endpoint))
            )
    })
    .bind("0.0.0.0:5004")?
    .run()
    .await
}

async fn create_room(
    sfu: web::Data<SFU>,
    form: web::Json<CreateRoomRequest>,
) -> impl Responder {
    match sfu.create_room(form.tenant_id, form.max_participants).await {
        Ok(room_id) => {
            info!("Room created: {}", room_id.0);
            let response = RoomResponse {
                room_id: room_id.0,
                tenant_id: form.tenant_id,
                max_participants: form.max_participants,
                created_at: Utc::now(),
            };
            HttpResponse::Created().json(response)
        }
        Err(e) => {
            error!("Failed to create room: {}", e);
            HttpResponse::BadRequest().json(json!({ "error": format!("{:?}", e) }))
        }
    }
}

async fn delete_room(
    sfu: web::Data<SFU>,
    room_id_path: web::Path<Uuid>,
) -> impl Responder {
    let room_id = room_id_path.into_inner();
    match sfu.delete_room(room_id).await {
        Ok(_) => {
            info!("Room deleted: {}", room_id);
            HttpResponse::NoContent().finish()
        }
        Err(e) => {
            error!("Failed to delete room: {}", e);
            HttpResponse::NotFound().json(json!({ "error": format!("{:?}", e) }))
        }
    }
}

async fn add_peer(
    sfu: web::Data<SFU>,
    form: web::Json<AddPeerRequest>,
) -> impl Responder {
    let transport = Arc::new(MockTransport);
    match sfu.add_peer(form.room_id, form.peer_id, transport).await {
        Ok(_) => {
            info!("Peer added to room: {}", form.peer_id);
            HttpResponse::NoContent().finish()
        }
        Err(e) => {
            error!("Failed to add peer: {}", e);
            HttpResponse::BadRequest().json(json!({ "error": format!("{:?}", e) }))
        }
    }
}

async fn add_track(
    _sfu: web::Data<SFU>,
    _form: web::Json<AddTrackRequest>,
) -> impl Responder {
    HttpResponse::NotImplemented().finish()
}

async fn get_room_stats(
    sfu: web::Data<SFU>,
    room_id_path: web::Path<Uuid>,
) -> impl Responder {
    let room_id = room_id_path.into_inner();
    match sfu.get_room_stats(room_id).await {
        Ok(stats) => {
            info!("Room stats retrieved: {}", room_id);
            HttpResponse::Ok().json(stats)
        }
        Err(e) => {
            error!("Failed to get room stats: {}", e);
            HttpResponse::NotFound().json(json!({ "error": format!("{:?}", e) }))
        }
    }
}

async fn publish_packet(
    sfu: web::Data<SFU>,
    packet: web::Json<RTPPacket>,
) -> impl Responder {
    let domain_packet = shared::media::RTPPacket {
        ssrc: packet.ssrc,
        sequence_number: packet.sequence_number,
        timestamp: packet.timestamp,
        payload_type: 96,
        payload: packet.payload.clone(),
        marker: packet.marker,
        extension: None,
    };

    match sfu.publish_rtp(domain_packet).await {
        Ok(_) => HttpResponse::Accepted().finish(),
        Err(e) => HttpResponse::BadRequest().json(json!({ "error": format!("{:?}", e) })),
    }
}

#[derive(Clone)]
struct MockPacketProcessor;

impl MockPacketProcessor {
    fn new() -> Self {
        Self
    }
}

impl shared::media::PacketProcessor for MockPacketProcessor {
    fn process_rtp(&self, _packet: shared::media::RTPPacket) -> Result<(), shared::media::PacketError> {
        Ok(())
    }
    fn process_rtcp(&self, _packet: shared::media::RTCPPacket) -> Result<(), shared::media::PacketError> {
        Ok(())
    }
    fn get_forwarding_strategy(&self, _track_id: Uuid) -> shared::media::ForwardingStrategy {
        shared::media::ForwardingStrategy::Unicast { peer_ids: vec![] }
    }
}

struct MockTransport;

impl shared::media::Transport for MockTransport {
    fn send_packet(&self, packet: shared::media::RTPPacket) -> Result<(), shared::media::TransportError> {
        info!("Transport: Sending packet SSRC={} Seq={}", packet.ssrc, packet.sequence_number);
        Ok(())
    }
    fn receive_packet(&self) -> Result<shared::media::RTPPacket, shared::media::TransportError> {
        Err(shared::media::TransportError::Timeout)
    }
    fn send_rtcp(&self, _packet: shared::media::RTCPPacket) -> Result<(), shared::media::TransportError> {
        Ok(())
    }
    fn receive_rtcp(&self) -> Result<shared::media::RTCPPacket, shared::media::TransportError> {
        Err(shared::media::TransportError::Timeout)
    }
}
