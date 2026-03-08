use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use log::{info};
use std::sync::Arc;

mod lib;
use crate::lib::{ControlPlane};
use shared::security::{AuthProvider, AuthError, User, Role};

struct SimpleAuthProvider;

impl AuthProvider for SimpleAuthProvider {
    fn authenticate(&self, _username: &str, _password: &str) -> Result<User, AuthError> {
        Err(AuthError::InvalidCredentials)
    }
    fn validate_token(&self, _token: &str) -> Result<User, AuthError> {
        Ok(User {
            id: uuid::Uuid::new_v4(),
            username: "admin".to_string(),
            email: "admin@drmp.com".to_string(),
            roles: vec![Role::SuperAdmin],
            tenant_id: None,
            created_at: chrono::Utc::now(),
        })
    }
    fn authorize(&self, _user: &User, _resource: &str, _action: &str) -> Result<bool, AuthError> {
        Ok(true)
    }
    fn create_token(&self, _user: &User) -> Result<String, AuthError> {
        Ok("dummy_token".to_string())
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:password@localhost/drmp".to_string());
    let db_pool = PgPool::connect(&database_url).await.expect("Failed to connect to database");
    
    let auth_provider = Box::new(SimpleAuthProvider);
    let control_plane = Arc::new(ControlPlane::new(db_pool, auth_provider));
    
    info!("Starting Control Plane service on port 8080");
    
    HttpServer::new(move || {
        let cp = control_plane.clone();
        App::new()
            .app_data(web::Data::new(cp))
            .service(web::resource("/api/rooms")
                .route(web::post().to(crate::lib::create_room_handler))
                .route(web::get().to(crate::lib::get_rooms_handler)))
            .service(web::resource("/api/rooms/{room_id}")
                .route(web::delete().to(crate::lib::delete_room_handler)))
            .service(web::resource("/api/peers")
                .route(web::post().to(crate::lib::create_peer_handler)))
            .service(web::resource("/api/tracks")
                .route(web::post().to(crate::lib::create_track_handler)))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
