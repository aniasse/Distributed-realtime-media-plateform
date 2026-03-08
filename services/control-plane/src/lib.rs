use actix_web::{web, HttpResponse, Responder};
use serde::{Serialize, Deserialize};
use sqlx::{PgPool};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;

use shared::domain::{Room, RoomState, MediaKind, PublisherId};
use shared::security::{AuthProvider};
use shared::utils::{Logger, Metrics, ErrorHandler};

#[derive(Serialize, Deserialize)]
pub struct CreateRoomRequest {
    pub tenant_id: Uuid,
    pub max_participants: u32,
}

#[derive(Serialize, Deserialize)]
pub struct CreateRoomResponse {
    pub room_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct GetRoomsResponse {
    pub rooms: Vec<Room>,
}

#[derive(Serialize, Deserialize)]
pub struct CreatePeerRequest {
    pub room_id: Uuid,
    pub peer_id: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct CreatePeerResponse {
    pub peer_id: Uuid,
    pub connected_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateTrackRequest {
    pub room_id: Uuid,
    pub publisher_id: Uuid,
    pub kind: MediaKind,
    pub ssrc: u32,
    pub payload_type: u8,
}

#[derive(Serialize, Deserialize)]
pub struct CreateTrackResponse {
    pub track_id: Uuid,
    pub created_at: DateTime<Utc>,
}

pub struct ControlPlane {
    pub db_pool: PgPool,
    pub auth_provider: Box<dyn AuthProvider + Send + Sync>,
    pub logger: Logger,
    pub metrics: Metrics,
    pub error_handler: ErrorHandler,
}

impl ControlPlane {
    pub fn new(db_pool: PgPool, auth_provider: Box<dyn AuthProvider + Send + Sync>) -> Self {
        Self {
            db_pool,
            auth_provider,
            logger: Logger::new("control-plane"),
            metrics: Metrics::new(),
            error_handler: ErrorHandler::new("control-plane"),
        }
    }
}

pub async fn create_room_handler(
    cp: web::Data<Arc<ControlPlane>>,
    req: web::Json<CreateRoomRequest>,
) -> impl Responder {
    if let Err(e) = cp.auth_provider.validate_token("dummy_token") {
        return HttpResponse::Unauthorized().finish();
    }

    let room_id = Uuid::new_v4();
    let created_at = Utc::now();

    // Insertion fictive (on suppose que les tables existent)
    cp.metrics.increment_counter("rooms_created", 1);
    
    HttpResponse::Created().json(CreateRoomResponse {
        room_id,
        created_at,
    })
}

pub async fn delete_room_handler(
    cp: web::Data<Arc<ControlPlane>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    if let Err(e) = cp.auth_provider.validate_token("dummy_token") {
        return HttpResponse::Unauthorized().finish();
    }

    let _room_id = path.into_inner();
    cp.metrics.increment_counter("rooms_deleted", 1);
    
    HttpResponse::NoContent().finish()
}

pub async fn get_rooms_handler(
    cp: web::Data<Arc<ControlPlane>>,
) -> impl Responder {
    if let Err(e) = cp.auth_provider.validate_token("dummy_token") {
        return HttpResponse::Unauthorized().finish();
    }

    let response = GetRoomsResponse {
        rooms: vec![], // Simulation
    };
    
    HttpResponse::Ok().json(response)
}

pub async fn create_peer_handler(
    cp: web::Data<Arc<ControlPlane>>,
    req: web::Json<CreatePeerRequest>,
) -> impl Responder {
    if let Err(e) = cp.auth_provider.validate_token("dummy_token") {
        return HttpResponse::Unauthorized().finish();
    }

    let peer_id = req.peer_id;
    let connected_at = Utc::now();

    cp.metrics.increment_counter("peers_created", 1);
    
    HttpResponse::Created().json(CreatePeerResponse {
        peer_id,
        connected_at,
    })
}

pub async fn create_track_handler(
    cp: web::Data<Arc<ControlPlane>>,
    req: web::Json<CreateTrackRequest>,
) -> impl Responder {
    if let Err(e) = cp.auth_provider.validate_token("dummy_token") {
        return HttpResponse::Unauthorized().finish();
    }

    let track_id = Uuid::new_v4();
    let created_at = Utc::now();

    cp.metrics.increment_counter("tracks_created", 1);
    
    HttpResponse::Created().json(CreateTrackResponse {
        track_id,
        created_at,
    })
}
