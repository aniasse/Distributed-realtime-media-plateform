use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::{Serialize, Deserialize};
use serde_json::json;
use log::{info, error};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};

mod lib;
use crate::lib::{MediaEdge};
use shared::security::{AuthProvider, AuthError, User, Role};
use shared::media::{Transport, RTPPacket, RTCPPacket, TransportError};

#[derive(Serialize, Deserialize)]
pub struct StartStreamRequest {
    pub stream_key: String,
    pub stream_type: String,
    pub resolution: Option<(u32, u32)>,
    pub bitrate: Option<u32>,
}

#[derive(Serialize, Deserialize)]
pub struct StreamInfo {
    pub stream_key: String,
    pub stream_type: String,
    pub status: String,
    pub connected_peers: u32,
    pub bitrate: u32,
    pub resolution: (u32, u32),
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct ListStreamsResponse {
    pub streams: Vec<StreamInfo>,
}

struct SimpleAuthProvider;

impl AuthProvider for SimpleAuthProvider {
    fn authenticate(&self, _username: &str, _password: &str) -> Result<User, AuthError> {
        Err(AuthError::InvalidCredentials)
    }
    fn validate_token(&self, _token: &str) -> Result<User, AuthError> {
        Ok(User {
            id: Uuid::new_v4(),
            username: "admin".to_string(),
            email: "admin@drmp.com".to_string(),
            roles: vec![Role::Admin],
            tenant_id: None,
            created_at: Utc::now(),
        })
    }
    fn authorize(&self, _user: &User, _resource: &str, _action: &str) -> Result<bool, AuthError> {
        Ok(true)
    }
    fn create_token(&self, _user: &User) -> Result<String, AuthError> {
        Ok("dummy_token".to_string())
    }
    fn validate_stream_key(&self, _key: &str) -> bool {
        true
    }
}

struct MockTransport;

impl Transport for MockTransport {
    fn send_packet(&self, _packet: RTPPacket) -> Result<(), TransportError> {
        Ok(())
    }
    fn receive_packet(&self) -> Result<RTPPacket, TransportError> {
        Err(TransportError::Timeout)
    }
    fn send_rtcp(&self, _packet: RTCPPacket) -> Result<(), TransportError> {
        Ok(())
    }
    fn receive_rtcp(&self) -> Result<RTCPPacket, TransportError> {
        Err(TransportError::Timeout)
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let transport = Box::new(MockTransport);
    let auth_provider = Box::new(SimpleAuthProvider);
    let media_edge = Arc::new(MediaEdge::new(transport, auth_provider));
    
    info!("Starting Media Edge service on port 8081");
    
    HttpServer::new(move || {
        let me = media_edge.clone();
        App::new()
            .app_data(web::Data::new(me))
            .service(
                web::resource("/api/streams")
                    .route(web::post().to(start_stream))
                    .route(web::get().to(list_streams))
            )
            .service(
                web::resource("/api/streams/{stream_key}")
                    .route(web::delete().to(stop_stream))
            )
    })
    .bind("0.0.0.0:8081")?
    .run()
    .await
}

async fn start_stream(
    media_edge: web::Data<Arc<MediaEdge>>,
    form: web::Json<StartStreamRequest>,
) -> impl Responder {
    match media_edge.start_stream(
        &form.stream_key,
        &form.stream_type,
        form.resolution,
        form.bitrate
    ).await {
        Ok(_) => {
            info!("Stream started: {}", form.stream_key);
            HttpResponse::Created().json(json!({ 
                "stream_key": form.stream_key,
                "stream_type": form.stream_type,
                "status": "started"
            }))
        }
        Err(e) => {
            error!("Failed to start stream: {}", e);
            HttpResponse::BadRequest().json(json!({ "error": format!("{:?}", e) }))
        }
    }
}

async fn stop_stream(
    media_edge: web::Data<Arc<MediaEdge>>,
    stream_key: web::Path<String>,
) -> impl Responder {
    match media_edge.stop_stream(&stream_key.into_inner()).await {
        Ok(_) => {
            info!("Stream stopped: {}", stream_key);
            HttpResponse::NoContent().finish()
        }
        Err(e) => {
            error!("Failed to stop stream: {}", e);
            HttpResponse::NotFound().json(json!({ "error": format!("{:?}", e) }))
        }
    }
}

async fn list_streams(
    media_edge: web::Data<Arc<MediaEdge>>,
) -> impl Responder {
    match media_edge.list_streams().await {
        Ok(streams) => {
            info!("Listing streams");
            let response = ListStreamsResponse {
                streams: streams.into_iter().map(|s| StreamInfo {
                    stream_key: s.stream_key,
                    stream_type: s.stream_type,
                    status: format!("{:?}", s.status),
                    connected_peers: s.connected_peers,
                    bitrate: s.bitrate,
                    resolution: s.resolution,
                    created_at: s.created_at,
                }).collect(),
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            error!("Failed to list streams: {}", e);
            HttpResponse::InternalServerError().json(json!({ "error": format!("{:?}", e) }))
        }
    }
}
