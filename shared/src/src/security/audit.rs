use actix_web::{dev::ServiceRequest, dev::ServiceResponse, HttpResponse, ResponseError};
use actix_web::http::header;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc};
use log::{info, error, warn, debug};
use crate::utils::{Logger, Metrics, ErrorHandler};
use crate::security::{AuthProvider, AuthError, Role, User, Permission, RBAC, RoleDefinition};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub id: String,
    pub event_type: SecurityEventType,
    pub user_id: Option<String>,
    pub ip_address: String,
    pub resource: String,
    pub action: String,
    pub details: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    AuthenticationSuccess,
    AuthenticationFailure,
    AuthorizationSuccess,
    AuthorizationFailure,
    RateLimitExceeded,
    InputValidationFailure,
    CredentialLeakAttempt,
    SecurityHeaderMissing,
    CorsPolicyViolation,
}

#[derive(Debug, Clone)]
pub struct SecurityAuditLogger {
    pub logger: Arc<Logger>,
    pub metrics: Arc<Metrics>,
    pub error_handler: Arc<ErrorHandler>,
    pub db_pool: Arc<sqlx::PgPool>,
}

impl SecurityAuditLogger {
    pub fn new(
        logger: Logger,
        metrics: Metrics,
        error_handler: ErrorHandler,
        db_pool: sqlx::PgPool,
    ) -> Self {
        Self {
            logger: Arc::new(logger),
            metrics: Arc::new(metrics),
            error_handler: Arc::new(error_handler),
            db_pool: Arc::new(db_pool),
        }
    }

    pub async fn log_event(&self, event: SecurityEvent) -> Result<(), sqlx::Error> {
        let event_id = uuid::Uuid::new_v4().to_string();
        let timestamp = Utc::now();
        
        // Insert into database
        let result = sqlx::query("
            INSERT INTO security_audit_log (id, event_type, user_id, ip_address, resource, action, details, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ")
        .bind(event_id)
        .bind(format!("{:?}", event.event_type))
        .bind(event.user_id.unwrap_or_default())
        .bind(event.ip_address)
        .bind(event.resource)
        .bind(event.action)
        .bind(event.details)
        .bind(timestamp)
        .execute(&self.db_pool)
        .await;

        match result {
            Ok(_) => {
                self.metrics.increment_counter("security_events_logged", 1);
                self.logger.info(&format!("Security event logged: {:?}", event.event_type));
                Ok(())
            }
            Err(e) => {
                self.error_handler.handle_error(&e, "log_security_event");
                Err(e)
            }
        }
    }

    pub async fn log_authentication_success(&self, user_id: &str, ip_address: &str, resource: &str) -> Result<(), sqlx::Error> {
        let event = SecurityEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: SecurityEventType::AuthenticationSuccess,
            user_id: Some(user_id.to_string()),
            ip_address: ip_address.to_string(),
            resource: resource.to_string(),
            action: "authenticate".to_string(),
            details: "Successful authentication".to_string(),
            timestamp: Utc::now(),
        };
        self.log_event(event).await
    }

    pub async fn log_authentication_failure(&self, ip_address: &str, resource: &str, reason: &str) -> Result<(), sqlx::Error> {
        let event = SecurityEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: SecurityEventType::AuthenticationFailure,
            user_id: None,
            ip_address: ip_address.to_string(),
            resource: resource.to_string(),
            action: "authenticate".to_string(),
            details: reason.to_string(),
            timestamp: Utc::now(),
        };
        self.log_event(event).await
    }

    pub async fn log_authorization_success(&self, user_id: &str, ip_address: &str, resource: &str, action: &str) -> Result<(), sqlx::Error> {
        let event = SecurityEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: SecurityEventType::AuthorizationSuccess,
            user_id: Some(user_id.to_string()),
            ip_address: ip_address.to_string(),
            resource: resource.to_string(),
            action: action.to_string(),
            details: "Successful authorization".to_string(),
            timestamp: Utc::now(),
        };
        self.log_event(event).await
    }

    pub async fn log_authorization_failure(&self, user_id: &str, ip_address: &str, resource: &str, action: &str, reason: &str) -> Result<(), sqlx::Error> {
        let event = SecurityEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: SecurityEventType::AuthorizationFailure,
            user_id: Some(user_id.to_string()),
            ip_address: ip_address.to_string(),
            resource: resource.to_string(),
            action: action.to_string(),
            details: reason.to_string(),
            timestamp: Utc::now(),
        };
        self.log_event(event).await
    }

    pub async fn log_rate_limit_exceeded(&self, ip_address: &str, resource: &str) -> Result<(), sqlx::Error> {
        let event = SecurityEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: SecurityEventType::RateLimitExceeded,
            user_id: None,
            ip_address: ip_address.to_string(),
            resource: resource.to_string(),
            action: "request".to_string(),
            details: "Rate limit exceeded".to_string(),
            timestamp: Utc::now(),
        };
        self.log_event(event).await
    }

    pub async fn log_input_validation_failure(&self, ip_address: &str, resource: &str, field: &str, reason: &str) -> Result<(), sqlx::Error> {
        let event = SecurityEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: SecurityEventType::InputValidationFailure,
            user_id: None,
            ip_address: ip_address.to_string(),
            resource: resource.to_string(),
            action: "input_validation".to_string(),
            details: format!("Validation failed for field '{}': {}", field, reason),
            timestamp: Utc::now(),
        };
        self.log_event(event).await
    }

    pub async fn log_credential_leak_attempt(&self, ip_address: &str, resource: &str, payload: &str) -> Result<(), sqlx::Error> {
        let event = SecurityEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: SecurityEventType::CredentialLeakAttempt,
            user_id: None,
            ip_address: ip_address.to_string(),
            resource: resource.to_string(),
            action: "credential_leak".to_string(),
            details: format!("Potential credential leak attempt: {}", payload),
            timestamp: Utc::now(),
        };
        self.log_event(event).await
    }

    pub async fn log_security_header_missing(&self, ip_address: &str, resource: &str, header: &str) -> Result<(), sqlx::Error> {
        let event = SecurityEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: SecurityEventType::SecurityHeaderMissing,
            user_id: None,
            ip_address: ip_address.to_string(),
            resource: resource.to_string(),
            action: "security_header".to_string(),
            details: format!("Missing security header: {}", header),
            timestamp: Utc::now(),
        };
        self.log_event(event).await
    }

    pub async fn log_cors_policy_violation(&self, ip_address: &str, origin: &str, resource: &str) -> Result<(), sqlx::Error> {
        let event = SecurityEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type: SecurityEventType::CorsPolicyViolation,
            user_id: None,
            ip_address: ip_address.to_string(),
            resource: resource.to_string(),
            action: "cors".to_string(),
            details: format!("CORS policy violation from origin: {}", origin),
            timestamp: Utc::now(),
        };
        self.log_event(event).await
    }
}

#[derive(Debug, Clone)]
pub struct SecurityEventMiddleware {
    pub audit_logger: Arc<SecurityAuditLogger>,
}

impl SecurityEventMiddleware {
    pub fn new(audit_logger: SecurityAuditLogger) -> Self {
        Self {
            audit_logger: Arc::new(audit_logger),
        }
    }

    pub async fn log_request(&self, req: &ServiceRequest) {
        let ip_address = req.connection_info().realip_remote_addr().unwrap_or("unknown").to_string();
        let resource = req.uri().path().to_string();
        let action = req.method().as_str().to_string();
        
        // Log basic request information
        self.audit_logger.logger.info(&format!("Request: {} {} from {}", action, resource, ip_address));
    }

    pub async fn log_response(&self, req: &ServiceRequest, res: &ServiceResponse) {
        let ip_address = req.connection_info().realip_remote_addr().unwrap_or("unknown").to_string();
        let resource = req.uri().path().to_string();
        let action = req.method().as_str().to_string();
        let status = res.status().as_u16();
        
        // Log response status
        self.audit_logger.logger.info(&format!("Response: {} {} {} from {}", status, action, resource, ip_address));
    }
}

#[derive(Debug, Clone)]
pub struct InputValidator {
    pub audit_logger: Arc<SecurityAuditLogger>,
}

impl InputValidator {
    pub fn new(audit_logger: SecurityAuditLogger) -> Self {
        Self {
            audit_logger: Arc::new(audit_logger),
        }
    }

    pub async fn validate_string(&self, field: &str, value: &str, max_length: usize) -> Result<(), String> {
        if value.len() > max_length {
            self.audit_logger.log_input_validation_failure(
                "unknown", 
                "input_validation", 
                field, 
                &format!("exceeds maximum length of {}", max_length)
            ).await.ok();
            return Err(format!("{} exceeds maximum length of {}", field, max_length));
        }

        if value.contains("<") || value.contains(">") || value.contains("'") || value.contains("\"") {
            self.audit_logger.log_input_validation_failure(
                "unknown", 
                "input_validation", 
                field, 
                "contains potentially dangerous characters"
            ).await.ok();
            return Err(format!("{} contains potentially dangerous characters", field));
        }

        Ok(())
    }

    pub async fn validate_email(&self, field: &str, value: &str) -> Result<(), String> {
        use regex::Regex;
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$).unwrap();
        
        if !email_regex.is_match(value) {
            self.audit_logger.log_input_validation_failure(
                "unknown", 
                "input_validation", 
                field, 
                "invalid email format"
            ).await.ok();
            return Err(format!("{} is not a valid email address", field));
        }

        Ok(())
    }

    pub async fn validate_uuid(&self, field: &str, value: &str) -> Result<(), String> {
        if let Err(_) = uuid::Uuid::parse_str(value) {
            self.audit_logger.log_input_validation_failure(
                "unknown", 
                "input_validation", 
                field, 
                "invalid UUID format"
            ).await.ok();
            return Err(format!("{} is not a valid UUID", field));
        }

        Ok(())
    }

    pub async fn validate_password(&self, field: &str, value: &str) -> Result<(), String> {
        if value.len() < 8 {
            self.audit_logger.log_input_validation_failure(
                "unknown", 
                "input_validation", 
                field, 
                "password must be at least 8 characters long"
            ).await.ok();
            return Err("Password must be at least 8 characters long".to_string());
        }

        let has_uppercase = value.chars().any(|c| c.is_uppercase());
        let has_lowercase = value.chars().any(|c| c.is_lowercase());
        let has_digit = value.chars().any(|c| c.is_digit(10));
        
        if !(has_uppercase && has_lowercase && has_digit) {
            self.audit_logger.log_input_validation_failure(
                "unknown", 
                "input_validation", 
                field, 
                "password must contain uppercase, lowercase, and digit"
            ).await.ok();
            return Err("Password must contain uppercase, lowercase, and digit".to_string());
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub allow_credentials: bool,
    pub max_age: u32,
}

#[derive(Debug, Clone)]
pub struct CorsMiddleware {
    pub config: CorsConfig,
    pub audit_logger: Arc<SecurityAuditLogger>,
}

impl CorsMiddleware {
    pub fn new(config: CorsConfig, audit_logger: SecurityAuditLogger) -> Self {
        Self {
            config,
            audit_logger: Arc::new(audit_logger),
        }
    }

    pub fn is_origin_allowed(&self, origin: &str) -> bool {
        for allowed_origin in &self.config.allowed_origins {
            if allowed_origin == "*" || allowed_origin == origin {
                return true;
            }
        }
        false
    }

    pub fn add_cors_headers(&self, req: &ServiceRequest, resp: &mut ServiceResponse) {
        if let Some(origin) = req.headers().get(header::ORIGIN) {
            let origin = origin.to_str().unwrap_or("");
            if self.is_origin_allowed(origin) {
                resp.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.parse().unwrap());
                if self.config.allow_credentials {
                    resp.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, "true".parse().unwrap());
                }
                resp.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_METHODS, 
                    self.config.allowed_methods.join(",").parse().unwrap());
                resp.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_HEADERS, 
                    self.config.allowed_headers.join(",").parse().unwrap());
                resp.headers_mut().insert(header::ACCESS_CONTROL_MAX_AGE, self.config.max_age.to_string().parse().unwrap());
            } else {
                self.audit_logger.log_cors_policy_violation(
                    req.connection_info().realip_remote_addr().unwrap_or("unknown"),
                    origin,
                    req.uri().path()
                ).await.ok();
            }
        }
    }
}

pub struct SecurityHeadersMiddleware {
    pub audit_logger: Arc<SecurityAuditLogger>,
}

impl SecurityHeadersMiddleware {
    pub fn new(audit_logger: SecurityAuditLogger) -> Self {
        Self {
            audit_logger: Arc::new(audit_logger),
        }
    }

    pub fn add_security_headers(&self, req: &ServiceRequest, resp: &mut ServiceResponse) {
        // Content Security Policy
        resp.headers_mut().insert(header::CONTENT_SECURITY_POLICY, 
            "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self';".parse().unwrap());
        
        // Other security headers
        resp.headers_mut().insert(header::X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());
        resp.headers_mut().insert(header::X_FRAME_OPTIONS, "DENY".parse().unwrap());
        resp.headers_mut().insert(header::X_XSS_PROTECTION, "1; mode=block".parse().unwrap());
        resp.headers_mut().insert(header::REFERRER_POLICY, "strict-origin-when-cross-origin".parse().unwrap());
        resp.headers_mut().insert(header::PERMISSIONS_POLICY, 
            "accelerometer=(), autoplay=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=()".parse().unwrap());
        
        // Check if headers were added successfully
        let headers = ["content-security-policy", "x-content-type-options", "x-frame-options", "x-xss-protection", "referrer-policy", "permissions-policy"];
        for header in headers {
            if !resp.headers().contains_key(header) {
                self.audit_logger.log_security_header_missing(
                    req.connection_info().realip_remote_addr().unwrap_or("unknown"),
                    req.uri().path(),
                    header
                ).await.ok();
            }
        }
    }
}