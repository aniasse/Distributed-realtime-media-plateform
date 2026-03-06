use actix_web::{web, App, HttpServer, Responder, HttpResponse, HttpMessage, http::StatusCode};
use actix_rt::System;
use sqlx::PgPool;
use serde::{Serialize, Deserialize};
use log::{info, error, debug};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::lib::{SFU, RoomError, PacketError};

#[derive(Serialize, Deserialize)]
pub struct CreateRoomRequest {
    pub tenant_id: uuid::Uuid,
    pub max_participants: u32,
}

#[derive(Serialize, Deserialize)]
pub struct RoomResponse {
    pub room_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub max_participants: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct AddPeerRequest {
    pub room_id: uuid::Uuid,
    pub peer_id: uuid::Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct AddTrackRequest {
    pub room_id: uuid::Uuid,
    pub track: Track,
}

#[derive(Serialize, Deserialize)]
pub struct Track {
    pub id: uuid::Uuid,
    pub publisher_id: uuid::Uuid,
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
    pub room_id: uuid::Uuid,
    pub peer_count: u32,
    pub active_peers: u32,
    pub track_count: u32,
    pub max_participants: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let packet_processor = MockPacketProcessor::new();
    let sfu = SFU::new(packet_processor);
    
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
                created_at: chrono::Utc::now(),
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
    room_id: web::Path<uuid::Uuid>,
) -> impl Responder {
    match sfu.delete_room(room_id.into_inner()).await {
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
    match sfu.add_peer(form.room_id, form.peer_id).await {
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
    sfu: web::Data<SFU>,
    form: web::Json<AddTrackRequest>,
) -> impl Responder {
    let track = form.track.clone();
    track.id = uuid::Uuid::new_v4();
    
    match sfu.add_track(form.room_id, track).await {
        Ok(_) => {
            info!("Track added to room");
            HttpResponse::NoContent().finish()
        }
        Err(e) => {
            error!("Failed to add track: {}", e);
            HttpResponse::BadRequest().json(json!({ "error": format!("{:?}", e) }))
        }
    }
}

async fn get_room_stats(
    sfu: web::Data<SFU>,
    room_id: web::Path<uuid::Uuid>,
) -> impl Responder {
    match sfu.get_room_stats(room_id.into_inner()).await {
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

struct MockPacketProcessor;

impl MockPacketProcessor {
    fn new() -> Self {
        Self
    }
}

impl crate::shared::media::PacketProcessor for MockPacketProcessor {
    fn process_rtp(&self, _packet: crate::shared::media::RTPPacket) -> Result<(), crate::shared::media::PacketError> {
        Ok(())
    }
    fn process_rtcp(&self, _packet: crate::shared::media::RTCPPacket) -> Result<(), crate::shared::media::PacketError> {
        Ok(())
    }
    fn get_forwarding_strategy(&self, _track_id: uuid::Uuid) -> crate::shared::media::ForwardingStrategy {
        crate::shared::media::ForwardingStrategy::Unicast { peer_ids: vec![] }
    }
}