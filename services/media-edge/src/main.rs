use actix_web::{web, App, HttpServer, Responder, HttpResponse, HttpMessage, http::StatusCode};
use actix_rt::System;
use serde::{Serialize, Deserialize};
use log::{info, error, debug};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::lib::{MediaEdge, MediaError};

#[derive(Serialize, Deserialize)]
pub struct StartStreamRequest {
    pub stream_key: String,
    pub stream_type: String,
    pub resolution: Option<(u32, u32)>,
    pub bitrate: Option<u32>,
}

#[derive(Serialize, Deserialize)]
pub struct StopStreamRequest {
    pub stream_key: String,
}

#[derive(Serialize, Deserialize)]
pub struct StreamInfo {
    pub stream_key: String,
    pub stream_type: String,
    pub status: String,
    pub connected_peers: u32,
    pub bitrate: u32,
    pub resolution: (u32, u32),
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct ListStreamsResponse {
    pub streams: Vec<StreamInfo>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let media_edge = MediaEdge::new();
    
    info!("Starting Media Edge service on ports 1935, 8081");
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(media_edge.clone()))
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
    media_edge: web::Data<MediaEdge>,
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
    media_edge: web::Data<MediaEdge>,
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
    media_edge: web::Data<MediaEdge>,
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