use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::{Serialize, Deserialize};
use serde_json::json;
use log::{info};
use std::sync::Arc;

mod lib;
use crate::lib::{Gateway};
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
    
    let auth_provider = Box::new(SimpleAuthProvider);
    let gateway = Arc::new(Gateway::new(auth_provider));
    
    info!("Starting Gateway service on port 8888");
    
    HttpServer::new(move || {
        let gateway = gateway.clone();
        App::new()
            .app_data(web::Data::new(gateway))
            .service(
                web::resource("/api/status")
                    .route(web::get().to(get_status))
            )
            .service(
                web::resource("/api/health")
                    .route(web::get().to(health_check))
            )
            .service(
                web::resource("/api/{service}/{endpoint}")
                    .route(web::post().to(crate::lib::route_request_handler))
            )
    })
    .bind("0.0.0.0:8888")?
    .run()
    .await
}

async fn get_status() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "service": "gateway",
        "status": "healthy",
        "version": "0.1.0"
    }))
}

async fn health_check(gateway: web::Data<Arc<Gateway>>) -> impl Responder {
    gateway.health_check().await
}
