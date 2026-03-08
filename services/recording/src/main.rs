use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::{Serialize, Deserialize};
use serde_json::json;
use log::{info, error};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};

mod lib;
use crate::lib::{RecordingService, FileStorageBackend};

#[derive(Serialize, Deserialize)]
pub struct StartRecordingRequest {
    pub room_id: Uuid,
    pub recording_name: String,
    pub recording_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct StopRecordingRequest {
    pub room_id: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct RecordingInfo {
    pub id: Uuid,
    pub room_id: Uuid,
    pub recording_name: String,
    pub recording_type: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub duration: Option<u32>,
}

#[derive(Serialize, Deserialize)]
pub struct ListRecordingsResponse {
    pub recordings: Vec<RecordingInfo>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let storage = Box::new(FileStorageBackend { base_path: "./recordings".to_string() });
    let recording_service = Arc::new(RecordingService::new(storage));
    
    info!("Starting Recording service on port 8082");
    
    HttpServer::new(move || {
        let rs = recording_service.clone();
        App::new()
            .app_data(web::Data::new(rs))
            .service(
                web::resource("/api/rooms")
                    .route(web::post().to(start_recording))
            )
            .service(
                web::resource("/api/rooms/{room_id}")
                    .route(web::delete().to(stop_recording))
            )
            .service(
                web::resource("/api/recordings")
                    .route(web::get().to(list_recordings))
            )
    })
    .bind("0.0.0.0:8082")?
    .run()
    .await
}

async fn start_recording(
    recording_service: web::Data<Arc<RecordingService>>,
    form: web::Json<StartRecordingRequest>,
) -> impl Responder {
    match recording_service.start_recording(
        form.room_id,
        &form.recording_name,
        &form.recording_type
    ).await {
        Ok(recording_id) => {
            info!("Recording started: {} for room {}", recording_id, form.room_id);
            HttpResponse::Created().json(json!({ 
                "recording_id": recording_id,
                "room_id": form.room_id,
                "recording_name": form.recording_name,
                "recording_type": form.recording_type,
                "status": "started"
            }))
        }
        Err(e) => {
            error!("Failed to start recording: {}", e);
            HttpResponse::BadRequest().json(json!({ "error": format!("{:?}", e) }))
        }
    }
}

async fn stop_recording(
    recording_service: web::Data<Arc<RecordingService>>,
    room_id: web::Path<Uuid>,
) -> impl Responder {
    match recording_service.stop_recording(room_id.into_inner()).await {
        Ok(duration) => {
            info!("Recording stopped for room {}", room_id);
            HttpResponse::Ok().json(json!({ 
                "room_id": room_id,
                "duration_seconds": duration,
                "status": "stopped"
            }))
        }
        Err(e) => {
            error!("Failed to stop recording: {}", e);
            HttpResponse::NotFound().json(json!({ "error": format!("{:?}", e) }))
        }
    }
}

async fn list_recordings(
    recording_service: web::Data<Arc<RecordingService>>,
) -> impl Responder {
    match recording_service.list_recordings().await {
        Ok(recordings) => {
            info!("Listing recordings");
            let response = ListRecordingsResponse {
                recordings: recordings.into_iter().map(|r| RecordingInfo {
                    id: r.id,
                    room_id: r.room_id,
                    recording_name: r.recording_name,
                    recording_type: r.recording_type,
                    status: r.status,
                    created_at: r.created_at,
                    duration: r.duration,
                }).collect(),
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            error!("Failed to list recordings: {}", e);
            HttpResponse::InternalServerError().json(json!({ "error": format!("{:?}", e) }))
        }
    }
}
