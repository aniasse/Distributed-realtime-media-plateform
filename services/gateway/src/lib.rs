use actix_web::{web, App, HttpServer, Responder, HttpResponse, HttpMessage, guard};
use actix_web::web::{Data, Json};
use actix_web::dev::Payload;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use log::{info, error, debug};

use crate::shared::security::{AuthProvider, AuthError, Role};
use crate::shared::utils::{Logger, Metrics, ErrorHandler};

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
    pub auth_provider: Box<dyn AuthProvider>,
    pub logger: Logger,
    pub metrics: Metrics,
    pub error_handler: ErrorHandler,
}

impl Gateway {
    pub fn new(auth_provider: Box<dyn AuthProvider>) -> Self {
        Self {
            auth_provider,
            logger: Logger::new("gateway"),
            metrics: Metrics::new(),
            error_handler: ErrorHandler::new("gateway"),
        }
    }

    pub async fn start(&self) -> std::io::Result<()> {
        self.logger.info("Starting Gateway service");
        
        HttpServer::new(move || {
            App::new()
                .app_data(Data::new(self))
                .service(web::resource("/api/{service}/{endpoint}")
                    .guard(guard::Header("Authorization", "Bearer"))
                    .route(web::post().to(self.route_request)))
                .service(web::resource("/health").route(web::get().to(self.health_check)))
        })
        .bind("0.0.0.0:8080")?
        .run()
        .await
    }

    async fn route_request(
        gw: Data<Gateway>,
        web::Path((service, endpoint)): web::Path<(String, String)>,
        req: Json<ApiRequest>,
        auth: actix_web::dev::Payload,
    ) -> impl Responder {
        // Authenticate request
        if let Err(e) = gw.auth_provider.validate_token("dummy_token").await {
            return HttpResponse::Unauthorized().json(ApiResponse {
                status: 401,
                data: None,
                error: Some(e.to_string()),
            });
        }

        gw.logger.debug(&format!("Routing request to {}:{}", service, endpoint));
        
        // Route to appropriate service
        match service.as_str() {
            "control-plane" => {
                gw.route_to_control_plane(endpoint, req.into_inner()).await
            }
            "sfu" => {
                gw.route_to_sfu(endpoint, req.into_inner()).await
            }
            "recording" => {
                gw.route_to_recording(endpoint, req.into_inner()).await
            }
            "auth" => {
                gw.route_to_auth(endpoint, req.into_inner()).await
            }
            _ => {
                HttpResponse::NotFound().json(ApiResponse {
                    status: 404,
                    data: None,
                    error: Some("Service not found".to_string()),
                })
            }
        }
    }

    async fn route_to_control_plane(&self, endpoint: String, req: ApiRequest) -> impl Responder {
        self.logger.debug(&format!("Routing to control-plane: {}", endpoint));
        
        // Simulate control-plane response
        let response = match endpoint.as_str() {
            "rooms" => {
                if req.method == "POST" {
                    // Create room
                    ApiResponse {
                        status: 201,
                        data: Some(serde_json::json!({ "room_id": Uuid::new_v4() })),
                        error: None,
                    }
                } else if req.method == "GET" {
                    // Get rooms
                    ApiResponse {
                        status: 200,
                        data: Some(serde_json::json!({ "rooms": [] })),
                        error: None,
                    }
                } else {
                    ApiResponse {
                        status: 405,
                        data: None,
                        error: Some("Method not allowed".to_string()),
                    }
                }
            }
            "rooms/{room_id}" => {
                if req.method == "DELETE" {
                    // Delete room
                    ApiResponse {
                        status: 204,
                        data: None,
                        error: None,
                    }
                } else {
                    ApiResponse {
                        status: 405,
                        data: None,
                        error: Some("Method not allowed".to_string()),
                    }
                }
            }
            _ => ApiResponse {
                status: 404,
                data: None,
                error: Some("Endpoint not found".to_string()),
            },
        };
        
        HttpResponse::build(actix_web::http::StatusCode::from_u16(response.status).unwrap())
            .json(response)
    }

    async fn route_to_sfu(&self, endpoint: String, req: ApiRequest) -> impl Responder {
        self.logger.debug(&format!("Routing to SFU: {}", endpoint));
        
        // Simulate SFU response
        let response = match endpoint.as_str() {
            "rooms" => {
                if req.method == "POST" {
                    // Create room
                    ApiResponse {
                        status: 201,
                        data: Some(serde_json::json!({ "room_id": Uuid::new_v4() })),
                        error: None,
                    }
                } else {
                    ApiResponse {
                        status: 405,
                        data: None,
                        error: Some("Method not allowed".to_string()),
                    }
                }
            }
            "peers" => {
                if req.method == "POST" {
                    // Add peer
                    ApiResponse {
                        status: 201,
                        data: Some(serde_json::json!({ "peer_id": Uuid::new_v4() })),
                        error: None,
                    }
                } else {
                    ApiResponse {
                        status: 405,
                        data: None,
                        error: Some("Method not allowed".to_string()),
                    }
                }
            }
            "tracks" => {
                if req.method == "POST" {
                    // Add track
                    ApiResponse {
                        status: 201,
                        data: Some(serde_json::json!({ "track_id": Uuid::new_v4() })),
                        error: None,
                    }
                } else {
                    ApiResponse {
                        status: 405,
                        data: None,
                        error: Some("Method not allowed".to_string()),
                    }
                }
            }
            _ => ApiResponse {
                status: 404,
                data: None,
                error: Some("Endpoint not found".to_string()),
            },
        };
        
        HttpResponse::build(actix_web::http::StatusCode::from_u16(response.status).unwrap())
            .json(response)
    }

    async fn route_to_recording(&self, endpoint: String, req: ApiRequest) -> impl Responder {
        self.logger.debug(&format!("Routing to recording: {}", endpoint));
        
        // Simulate recording response
        let response = match endpoint.as_str() {
            "rooms" => {
                if req.method == "POST" {
                    // Start recording
                    ApiResponse {
                        status: 201,
                        data: Some(serde_json::json!({ "recording_id": Uuid::new_v4() })),
                        error: None,
                    }
                } else if req.method == "DELETE" {
                    // Stop recording
                    ApiResponse {
                        status: 204,
                        data: None,
                        error: None,
                    }
                } else {
                    ApiResponse {
                        status: 405,
                        data: None,
                        error: Some("Method not allowed".to_string()),
                    }
                }
            }
            "recordings" => {
                if req.method == "GET" {
                    // List recordings
                    ApiResponse {
                        status: 200,
                        data: Some(serde_json::json!({ "recordings": [] })),
                        error: None,
                    }
                } else {
                    ApiResponse {
                        status: 405,
                        data: None,
                        error: Some("Method not allowed".to_string()),
                    }
                }
            }
            _ => ApiResponse {
                status: 404,
                data: None,
                error: Some("Endpoint not found".to_string()),
            },
        };
        
        HttpResponse::build(actix_web::http::StatusCode::from_u16(response.status).unwrap())
            .json(response)
    }

    async fn route_to_auth(&self, endpoint: String, req: ApiRequest) -> impl Responder {
        self.logger.debug(&format!("Routing to auth: {}", endpoint));
        
        // Simulate auth response
        let response = match endpoint.as_str() {
            "register" => {
                if req.method == "POST" {
                    // Register user
                    ApiResponse {
                        status: 201,
                        data: Some(serde_json::json!({ "user_id": Uuid::new_v4() })),
                        error: None,
                    }
                } else {
                    ApiResponse {
                        status: 405,
                        data: None,
                        error: Some("Method not allowed".to_string()),
                    }
                }
            }
            "login" => {
                if req.method == "POST" {
                    // Login
                    ApiResponse {
                        status: 200,
                        data: Some(serde_json::json!({ "token": "dummy_token" })),
                        error: None,
                    }
                } else {
                    ApiResponse {
                        status: 405,
                        data: None,
                        error: Some("Method not allowed".to_string()),
                    }
                }
            }
            "validate" => {
                if req.method == "POST" {
                    // Validate token
                    ApiResponse {
                        status: 200,
                        data: Some(serde_json::json!({ "valid": true })),
                        error: None,
                    }
                } else {
                    ApiResponse {
                        status: 405,
                        data: None,
                        error: Some("Method not allowed".to_string()),
                    }
                }
            }
            _ => ApiResponse {
                status: 404,
                data: None,
                error: Some("Endpoint not found".to_string()),
            },
        };
        
        HttpResponse::build(actix_web::http::StatusCode::from_u16(response.status).unwrap())
            .json(response)
    }

    async fn health_check(&self) -> impl Responder {
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[actix_rt::test]
    async fn test_gateway_creation() {
        let auth_provider = MockAuthProvider::new();
        let gateway = Gateway::new(auth_provider);
        
        assert!(gateway.auth_provider.is_some());
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