use actix_web::{web, App, HttpServer, Responder, HttpResponse, HttpMessage, http::StatusCode};
use actix_rt::System;
use sqlx::PgPool;
use serde::{Serialize, Deserialize};
use log::{info, error, debug};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::lib::{AuthService, AuthError};

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: uuid::Uuid,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

pub async fn register_user(
    db_pool: web::Data<PgPool>,
    form: web::Json<RegisterRequest>,
) -> impl Responder {
    let auth_service = AuthService::new(db_pool.clone());
    
    match auth_service.register_user(&form.username, &form.email, &form.password).await {
        Ok(user_id) => {
            info!("User registered: {}", form.username);
            HttpResponse::Created()
                .json(json!({ "message": "User registered successfully", "user_id": user_id }))
        }
        Err(e) => {
            error!("Failed to register user: {}", e);
            HttpResponse::BadRequest().json(json!({ "error": "Registration failed" }))
        }
    }
}

pub async fn login(
    db_pool: web::Data<PgPool>,
    form: web::Json<LoginRequest>,
) -> impl Responder {
    let auth_service = AuthService::new(db_pool.clone());
    
    match auth_service.authenticate(&form.username, &form.password).await {
        Ok(user) => {
            info!("User logged in: {}", form.username);
            let token = auth_service.create_token(&user, vec![]).await.unwrap();
            
            let login_response = LoginResponse {
                token,
                user: User {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                    roles: user.roles.into_iter().map(|r| format!("{:?}", r)).collect(),
                    created_at: user.created_at,
                },
            };
            
            HttpResponse::Ok().json(login_response)
        }
        Err(e) => {
            error!("Failed to authenticate user: {}", e);
            HttpResponse::Unauthorized().json(json!({ "error": "Invalid credentials" }))
        }
    }
}

pub async fn validate_token(
    db_pool: web::Data<PgPool>,
    token: web::Header("Authorization"),
) -> impl Responder {
    let auth_service = AuthService::new(db_pool.clone());
    
    let token = token.to_string().replace("Bearer ", "");
    
    match auth_service.validate_token(&token).await {
        Ok(user) => {
            info!("Token validated for user: {}", user.username);
            HttpResponse::Ok().json(json!({ "valid": true, "user": user.username }))
        }
        Err(e) => {
            error!("Token validation failed: {}", e);
            HttpResponse::Unauthorized().json(json!({ "valid": false, "error": "Invalid token" }))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPool::connect(&database_url).await.expect("Failed to connect to database");
    
    let auth_service = AuthService::new(db_pool.clone());
    
    info!("Starting Auth service on port 8081");
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .service(
                web::resource("/api/register")
                    .route(web::post().to(register_user))
            )
            .service(
                web::resource("/api/login")
                    .route(web::post().to(login))
            )
            .service(
                web::resource("/api/validate")
                    .route(web::post().to(validate_token))
            )
    })
    .bind("0.0.0.0:8081")?
    .run()
    .await
}