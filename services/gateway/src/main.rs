use actix_web::{web, App, HttpServer, Responder, HttpResponse, HttpMessage, http::StatusCode};
use actix_rt::System;
use serde::{Serialize, Deserialize};
use log::{info, error, debug};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::lib::{Gateway, ServiceInfo};

#[derive(Serialize, Deserialize)]
pub struct ServiceStatus {
    pub service_name: String,
    pub status: String,
    pub endpoint: String,
    pub version: String,
    pub uptime: u64,
}

#[derive(Serialize, Deserialize)]
pub struct ServicesStatus {
    pub services: Vec<ServiceStatus>,
    pub overall_status: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let gateway = Gateway::new();
    
    info!("Starting Gateway service on port 8888");
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(gateway.clone()))
            .service(
                web::resource("/api/status")
                    .route(web::get().to(get_status))
            )
            .service(
                web::resource("/api/services")
                    .route(web::get().to(get_services))
            )
            .service(
                web::resource("/api/health")
                    .route(web::get().to(health_check))
            )
    })
    .bind("0.0.0.0:8888")?
    .run()
    .await
}

async fn get_status(
    gateway: web::Data<Gateway>,
) -> impl Responder {
    match gateway.get_status().await {
        Ok(status) => {
            info!("Gateway status retrieved");
            HttpResponse::Ok().json(status)
        }
        Err(e) => {
            error!("Failed to get gateway status: {}", e);
            HttpResponse::InternalServerError().json(json!({ "error": format!("{:?}", e) }))
        }
    }
}

async fn get_services(
    gateway: web::Data<Gateway>,
) -> impl Responder {
    match gateway.get_services().await {
        Ok(services) => {
            info!("Services retrieved");
            let service_statuses: Vec<ServiceStatus> = services
                .into_iter()
                .map(|s| ServiceStatus {
                    service_name: s.service_name,
                    status: s.status,
                    endpoint: s.endpoint,
                    version: s.version,
                    uptime: s.uptime,
                })
                .collect();
            
            let overall_status = if service_statuses.iter().all(|s| s.status == "healthy") {
                "healthy".to_string()
            } else {
                "degraded".to_string()
            };
            
            let response = ServicesStatus {
                services: service_statuses,
                overall_status,
            };
            
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            error!("Failed to get services: {}", e);
            HttpResponse::InternalServerError().json(json!({ "error": format!("{:?}", e) }))
        }
    }
}

async fn health_check(
    gateway: web::Data<Gateway>,
) -> impl Responder {
    match gateway.health_check().await {
        Ok(healthy) => {
            if healthy {
                info!("Health check passed");
                HttpResponse::Ok().json(json!({ "status": "healthy" }))
            } else {
                HttpResponse::ServiceUnavailable().json(json!({ "status": "unhealthy" }))
            }
        }
        Err(e) => {
            error!("Health check failed: {}", e);
            HttpResponse::ServiceUnavailable().json(json!({ "status": "unhealthy", "error": format!("{:?}", e) }))
        }
    }
}