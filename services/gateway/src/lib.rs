use actix_web::{web, HttpResponse, Responder};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use log::{debug, info};
use std::sync::Arc;

use shared::security::{AuthProvider};
use shared::utils::{Logger, Metrics, ErrorHandler};

#[derive(Serialize, Deserialize)]
pub struct ApiRequest {
    pub endpoint: String,
    pub method: String,
    pub body: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct ApiResponse {
    pub status: u16,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

pub struct Gateway {
    pub auth_provider: Box<dyn AuthProvider + Send + Sync>,
    pub logger: Logger,
    pub metrics: Metrics,
    pub error_handler: ErrorHandler,
}

impl Gateway {
    pub fn new(auth_provider: Box<dyn AuthProvider + Send + Sync>) -> Self {
        Self {
            auth_provider,
            logger: Logger::new("gateway"),
            metrics: Metrics::new(),
            error_handler: ErrorHandler::new("gateway"),
        }
    }

    pub async fn health_check(&self) -> impl Responder {
        self.logger.info("Health check requested");
        
        HttpResponse::Ok().json(serde_json::json!({ 
            "status": "healthy",
            "services": {
                "control-plane": "healthy",
                "sfu": "healthy", 
                "recording": "healthy",
                "auth": "healthy"
            }
        }))
    }
}

pub async fn route_request_handler(
    gw: web::Data<Arc<Gateway>>,
    path: web::Path<(String, String)>,
    req: web::Json<ApiRequest>,
) -> impl Responder {
    let (service, endpoint) = path.into_inner();
    
    // Simple authentication check (simulation)
    if let Err(e) = gw.auth_provider.validate_token("dummy_token") {
        return HttpResponse::Unauthorized().json(ApiResponse {
            status: 401,
            data: None,
            error: Some(format!("{:?}", e)),
        });
    }

    gw.logger.debug(&format!("Routing request to {}:{}", service, endpoint));
    
    // Simulate routing
    let response = ApiResponse {
        status: 200,
        data: Some(serde_json::json!({ 
            "message": format!("Routed to {}:{}", service, endpoint),
            "request_data": req.into_inner()
        })),
        error: None,
    };
    
    HttpResponse::Ok().json(response)
}
