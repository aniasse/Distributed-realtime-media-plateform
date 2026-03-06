use actix_web::{web, App, HttpServer, Responder, HttpResponse, HttpMessage, http::StatusCode};
use actix_rt::System;
use sqlx::PgPool;
use serde::{Serialize, Deserialize};
use log::{info, error, debug};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::lib::{ControlPlane, Room, Peer, Track, RoomState};

#[derive(Serialize, Deserialize)]
pub struct CreateRoomRequest {
    pub tenant_id: uuid::Uuid,
    pub max_participants: u32,
    pub room_name: String,
    pub room_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct RoomResponse {
    pub room_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub room_name: String,
    pub room_type: String,
    pub max_participants: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub state: String,
}

#[derive(Serialize, Deserialize)]
pub struct ListRoomsResponse {
    pub rooms: Vec<RoomResponse>,
}

#[derive(Serialize, Deserialize)]
pub struct AddPeerRequest {
    pub room_id: uuid::Uuid,
    pub peer_id: uuid::Uuid,
    pub username: String,
    pub role: String,
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
pub struct DeleteRoomResponse {
    pub success: bool,
    pub message: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let db_pool = PgPool::connect("host=localhost user=postgres password=password database=drmp").await.expect("Failed to connect to database");
    let control_plane = ControlPlane::new(db_pool.clone());
    
    info!("Starting Control Plane service on port 8080");
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(control_plane.clone()))
            .app_data(web::Data::new(db_pool.clone()))
            .service(
                web::resource("/api/rooms")
                    .route(web::post().to(create_room))
                    .route(web::get().to(list_rooms))
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
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

async fn create_room(
    control_plane: web::Data<ControlPlane>,
    form: web::Json<CreateRoomRequest>,
) -> impl Responder {
    let room = Room {
        id: uuid::Uuid::new_v4(),
        tenant_id: form.tenant_id,
        room_name: form.room_name.clone(),
        room_type: form.room_type.clone(),
        max_participants: form.max_participants,
        created_at: chrono::Utc::now(),
        state: RoomState::Active,
        peers: Default::default(),
        tracks: Default::default(),
    };
    
    match control_plane.create_room(room).await {
        Ok(_) => {
            info!("Room created: {}", room.id);
            let response = RoomResponse {
                room_id: room.id,
                tenant_id: room.tenant_id,
                room_name: room.room_name,
                room_type: room.room_type,
                max_participants: room.max_participants,
                created_at: room.created_at,
                state: format!("{:?}", room.state),
            };
            HttpResponse::Created().json(response)
        }
        Err(e) => {
            error!("Failed to create room: {}", e);
            HttpResponse::BadRequest().json(json!({ "error": format!("{:?}", e) }))
        }
    }
}

async fn list_rooms(
    control_plane: web::Data<ControlPlane>,
) -> impl Responder {
    match control_plane.list_rooms().await {
        Ok(rooms) => {
            info!("Listing rooms");
            let response = ListRoomsResponse {
                rooms: rooms.into_iter().map(|r| RoomResponse {
                    room_id: r.id,
                    tenant_id: r.tenant_id,
                    room_name: r.room_name,
                    room_type: r.room_type,
                    max_participants: r.max_participants,
                    created_at: r.created_at,
                    state: format!("{:?}", r.state),
                }).collect(),
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            error!("Failed to list rooms: {}", e);
            HttpResponse::InternalServerError().json(json!({ "error": format!("{:?}", e) }))
        }
    }
}

async fn delete_room(
    control_plane: web::Data<ControlPlane>,
    room_id: web::Path<uuid::Uuid>,
) -> impl Responder {
    match control_plane.delete_room(room_id.into_inner()).await {
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
    control_plane: web::Data<ControlPlane>,
    form: web::Json<AddPeerRequest>,
) -> impl Responder {
    let peer = Peer {
        id: form.peer_id,
        username: form.username.clone(),
        role: form.role.clone(),n        connected_at: chrono::Utc::now(),
        tracks_subscribed: Default::default(),
        bandwidth_estimate: Default::default(),
        connection_state: Default::default(),
        ice_candidates: Default::default(),
        dtls_fingerprints: Default::default(),
    };
    
    match control_plane.add_peer(form.room_id, peer).await {
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
    control_plane: web::Data<ControlPlane>,
    form: web::Json<AddTrackRequest>,
) -> impl Responder {
    let track = form.track.clone();
    track.id = uuid::Uuid::new_v4();
    
    match control_plane.add_track(form.room_id, track).await {
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