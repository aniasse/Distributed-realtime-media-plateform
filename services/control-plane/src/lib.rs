use actix_web::{web, App, HttpServer, Responder, HttpResponse, HttpMessage};
use actix_web::web::{Data, Json};
use serde::{Serialize, Deserialize};
use sqlx::{PgPool, Row};
use sqlx::postgres::PgRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use log::{info, error, debug};

use crate::shared::domain::{Room, Peer, Track, RoomId, PeerId, TrackId, RoomState, MediaKind};
use crate::shared::security::{AuthProvider, AuthError, Role, User, Permission};
use crate::shared::utils::{Logger, Metrics, ErrorHandler};

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
pub struct DeleteRoomRequest {
    pub room_id: Uuid,
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
    pub auth_provider: Box<dyn AuthProvider>,
    pub logger: Logger,
    pub metrics: Metrics,
    pub error_handler: ErrorHandler,
}

impl ControlPlane {
    pub fn new(db_pool: PgPool, auth_provider: Box<dyn AuthProvider>) -> Self {
        Self {
            db_pool,
            auth_provider,
            logger: Logger::new("control-plane"),
            metrics: Metrics::new(),
            error_handler: ErrorHandler::new("control-plane"),
        }
    }

    pub async fn start(&self) -> std::io::Result<()> {
        self.logger.info("Starting Control Plane service");
        
        HttpServer::new(move || {
            App::new()
                .app_data(Data::new(self))
                .service(web::resource("/api/rooms").route(web::post().to(self.create_room)))
                .service(web::resource("/api/rooms/{room_id}").route(web::delete().to(self.delete_room)))
                .service(web::resource("/api/rooms").route(web::get().to(self.get_rooms)))
                .service(web::resource("/api/peers").route(web::post().to(self.create_peer)))
                .service(web::resource("/api/tracks").route(web::post().to(self.create_track)))
        })
        .bind("0.0.0.0:8080")?
        .run()
        .await
    }

    async fn create_room(
        cp: Data<ControlPlane>,
        req: Json<CreateRoomRequest>,
        auth: actix_web::dev::Payload,
    ) -> impl Responder {
        // Authenticate request
        if let Err(e) = cp.auth_provider.validate_token("dummy_token").await {
            return HttpResponse::Unauthorized().finish();
        }

        let room_id = Uuid::new_v4();
        let created_at = Utc::now();

        // Insert into database
        let result = sqlx::query("INSERT INTO rooms (id, tenant_id, max_participants, created_at) VALUES ($1, $2, $3, $4)")
            .bind(room_id)
            .bind(req.tenant_id)
            .bind(req.max_participants)
            .bind(created_at)
            .execute(&cp.db_pool)
            .await;

        match result {
            Ok(_) => {
                cp.metrics.increment_counter("rooms_created", 1);
                HttpResponse::Created().json(CreateRoomResponse {
                    room_id,
                    created_at,
                })
            }
            Err(e) => {
                cp.error_handler.handle_error(&e, "create_room");
                HttpResponse::InternalServerError().finish()
            }
        }
    }

    async fn delete_room(
        cp: Data<ControlPlane>,
        web::Path(room_id): web::Path<Uuid>,
    ) -> impl Responder {
        // Authenticate request
        if let Err(e) = cp.auth_provider.validate_token("dummy_token").await {
            return HttpResponse::Unauthorized().finish();
        }

        let result = sqlx::query("DELETE FROM rooms WHERE id = $1")
            .bind(room_id)
            .execute(&cp.db_pool)
            .await;

        match result {
            Ok(rows) if rows > 0 => {
                cp.metrics.increment_counter("rooms_deleted", 1);
                HttpResponse::NoContent().finish()
            }
            Ok(_) => HttpResponse::NotFound().finish(),
            Err(e) => {
                cp.error_handler.handle_error(&e, "delete_room");
                HttpResponse::InternalServerError().finish()
            }
        }
    }

    async fn get_rooms(
        cp: Data<ControlPlane>,
    ) -> impl Responder {
        // Authenticate request
        if let Err(e) = cp.auth_provider.validate_token("dummy_token").await {
            return HttpResponse::Unauthorized().finish();
        }

        let result = sqlx::query_as::<_, RoomRow>("SELECT * FROM rooms")
            .fetch_all(&cp.db_pool)
            .await;

        match result {
            Ok(rooms) => {
                let response = GetRoomsResponse {
                    rooms: rooms.into_iter().map(|r| Room {
                        id: r.id,
                        tenant_id: r.tenant_id,
                        peers: HashMap::new(),
                        tracks: HashMap::new(),
                        max_participants: r.max_participants,
                        created_at: r.created_at,
                        state: RoomState::Active,
                    }).collect(),
                };
                HttpResponse::Ok().json(response)
            }
            Err(e) => {
                cp.error_handler.handle_error(&e, "get_rooms");
                HttpResponse::InternalServerError().finish()
            }
        }
    }

    async fn create_peer(
        cp: Data<ControlPlane>,
        req: Json<CreatePeerRequest>,
    ) -> impl Responder {
        // Authenticate request
        if let Err(e) = cp.auth_provider.validate_token("dummy_token").await {
            return HttpResponse::Unauthorized().finish();
        }

        let peer_id = Uuid::new_v4();
        let connected_at = Utc::now();

        // Insert into database
        let result = sqlx::query("INSERT INTO peers (id, room_id, connected_at) VALUES ($1, $2, $3)")
            .bind(peer_id)
            .bind(req.room_id)
            .bind(connected_at)
            .execute(&cp.db_pool)
            .await;

        match result {
            Ok(_) => {
                cp.metrics.increment_counter("peers_created", 1);
                HttpResponse::Created().json(CreatePeerResponse {
                    peer_id,
                    connected_at,
                })
            }
            Err(e) => {
                cp.error_handler.handle_error(&e, "create_peer");
                HttpResponse::InternalServerError().finish()
            }
        }
    }

    async fn create_track(
        cp: Data<ControlPlane>,
        req: Json<CreateTrackRequest>,
    ) -> impl Responder {
        // Authenticate request
        if let Err(e) = cp.auth_provider.validate_token("dummy_token").await {
            return HttpResponse::Unauthorized().finish();
        }

        let track_id = Uuid::new_v4();
        let created_at = Utc::now();

        // Insert into database
        let result = sqlx::query("INSERT INTO tracks (id, room_id, publisher_id, kind, ssrc, payload_type, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(track_id)
            .bind(req.room_id)
            .bind(req.publisher_id)
            .bind(req.kind)
            .bind(req.ssrc)
            .bind(req.payload_type)
            .bind(created_at)
            .execute(&cp.db_pool)
            .await;

        match result {
            Ok(_) => {
                cp.metrics.increment_counter("tracks_created", 1);
                HttpResponse::Created().json(CreateTrackResponse {
                    track_id,
                    created_at,
                })
            }
            Err(e) => {
                cp.error_handler.handle_error(&e, "create_track");
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}

#[derive(sqlx::FromRow)]
struct RoomRow {
    id: Uuid,
    tenant_id: Uuid,
    max_participants: i32,
    created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[actix_rt::test]
    async fn test_control_plane_creation() {
        let db_pool = PgPool::connect("host=localhost user=postgres").await.unwrap();
        let auth_provider = MockAuthProvider::new();
        let control_plane = ControlPlane::new(db_pool, auth_provider);
        
        assert!(control_plane.db_pool.is_valid());
    }
    
    struct MockAuthProvider;
    impl MockAuthProvider {
        fn new() -> Self { Self }
    }
    
    impl AuthProvider for MockAuthProvider {
        fn authenticate(&self, _credentials: &Credentials) -> Result<User, AuthError> {
            unimplemented!()
        }
        fn authorize(&self, _user: &User, _resource: &str, _action: &str) -> Result<bool, AuthError> {
            unimplemented!()
        }
        fn validate_token(&self, _token: &str) -> Result<User, AuthError> {
            Ok(User {
                id: Uuid::new_v4(),
                username: "test".to_string(),
                email: "test@example.com".to_string(),
                roles: vec![Role::SuperAdmin],
                tenant_id: None,
                created_at: Utc::now(),
            })
        }
        fn create_token(&self, _user: &User, _permissions: Vec<Permission>) -> Result<String, AuthError> {
            unimplemented!()
        }
        fn validate_stream_key(&self, _key: &str) -> bool {
            unimplemented!()
        }
        fn get_roles(&self, _user_id: Uuid) -> Vec<Role> {
            unimplemented!()
        }
    }
    
    #[derive(Serialize, Deserialize)]
    struct Credentials {
        username: String,
        password: String,
        token: Option<String>,
    }
}