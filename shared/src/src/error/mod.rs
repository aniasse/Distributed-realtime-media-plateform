use std::fmt;
use std::error::Error as StdError;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use log::{error, warn, info, debug};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Error {
    pub code: String,
    pub message: String,
    pub description: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub service: String,
    pub severity: ErrorSeverity,
    pub retryable: bool,
    pub details: Option<serde_json::Value>,
    pub correlation_id: Option<String>,
    pub stack_trace: Option<String>,
    pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Error,
    Warning,
    Info,
}

impl Error {
    pub fn new(code: &str, message: &str, service: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            description: None,
            timestamp: Utc::now(),
            service: service.to_string(),
            severity: ErrorSeverity::Error,
            retryable: false,
            details: None,
            correlation_id: None,
            stack_trace: None,
            metadata: None,
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_correlation_id(mut self, correlation_id: &str) -> Self {
        self.correlation_id = Some(correlation_id.to_string());
        self
    }

    pub fn with_stack_trace(mut self, stack_trace: &str) -> Self {
        self.stack_trace = Some(stack_trace.to_string());
        self
    }

    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        if self.metadata.is_none() {
            self.metadata = Some(serde_json::Map::new());
        }
        self.metadata.as_mut().unwrap().insert(key.to_string(), value);
        self
    }

    pub fn wrap<E: StdError>(error: E, context: &str) -> Self {
        Self::new("DRMP-GEN-SYS-E-UNKNOWN-000", context, "unknown")
            .with_description(&format!("{}", error))
            .with_stack_trace(&format!("{}", error))
    }

    pub fn to_response(&self) -> serde_json::Value {
        serde_json::json!({
            "error": {
                "code": self.code,
                "message": self.message,
                "description": self.description,
                "timestamp": self.timestamp.to_rfc3339(),
                "service": self.service,
                "severity": format!("{:?}", self.severity).to_lowercase(),
                "retryable": self.retryable,
                "details": self.details,
                "correlation_id": self.correlation_id,
                "metadata": self.metadata
            }
        })
    }

    pub fn log(&self) {
        match self.severity {
            ErrorSeverity::Error => {
                error!("{}", self.to_log_message());
            }
            ErrorSeverity::Warning => {
                warn!("{}", self.to_log_message());
            }
            ErrorSeverity::Info => {
                info!("{}", self.to_log_message());
            }
        }
    }

    fn to_log_message(&self) -> String {
        let mut parts = vec![
            format!("[[1;31mERROR[0m] {} {}", self.code, self.message),
        ];
        
        if let Some(desc) = &self.description {
            parts.push(format!("Description: {}", desc));
        }
        
        if let Some(corr_id) = &self.correlation_id {
            parts.push(format!("Correlation ID: {}", corr_id));
        }
        
        if let Some(metadata) = &self.metadata {
            parts.push(format!("Metadata: {:?}", metadata));
        }
        
        parts.join(" ")
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl StdError for Error {}

pub mod codes {
    pub const DATABASE_CONNECTION_FAILED: &str = "DRMP-GEN-SYS-E-DB-001";
    pub const DATABASE_QUERY_TIMEOUT: &str = "DRMP-GEN-SYS-E-DB-002";
    pub const SERVICE_UNAVAILABLE: &str = "DRMP-GEN-SYS-E-NET-001";
    pub const CONNECTION_TIMEOUT: &str = "DRMP-GEN-SYS-E-NET-002";
    pub const INVALID_REQUEST_FORMAT: &str = "DRMP-GEN-VAL-E-REQ-001";
    pub const MISSING_REQUIRED_FIELDS: &str = "DRMP-GEN-VAL-E-REQ-002";
    pub const INVALID_TOKEN_FORMAT: &str = "DRMP-AUTH-AUTH-E-TOKEN-001";
    pub const TOKEN_EXPIRED: &str = "DRMP-AUTH-AUTH-E-TOKEN-002";
    pub const INSUFFICIENT_PERMISSIONS: &str = "DRMP-AUTH-AUTHZ-E-PERM-001";
    pub const CONNECTION_ESTABLISHMENT_FAILED: &str = "DRMP-SFU-MEDIA-E-CONN-001";
    pub const CONNECTION_LOST: &str = "DRMP-SFU-MEDIA-E-CONN-002";
    pub const MAX_CONNECTIONS_REACHED: &str = "DRMP-SFU-MEDIA-E-CONN-003";
    pub const STORAGE_DEVICE_UNAVAILABLE: &str = "DRMP-REC-MEDIA-E-STORE-001";
    pub const INSUFFICIENT_STORAGE_SPACE: &str = "DRMP-REC-MEDIA-E-STORE-002";
    pub const RECORDING_INITIALIZATION_FAILED: &str = "DRMP-REC-MEDIA-E-PROC-001";
    pub const RECORDING_ENCODING_FAILED: &str = "DRMP-REC-MEDIA-E-PROC-002";
    pub const RESOURCE_NOT_FOUND: &str = "DRMP-GEN-VAL-E-BUS-004";
    pub const RESOURCE_ALREADY_EXISTS: &str = "DRMP-GEN-VAL-E-BUS-003";
    pub const OPERATION_NOT_ALLOWED: &str = "DRMP-GEN-VAL-E-BUS-002";
}

pub mod builders {
    use super::*;

    pub fn database_connection_failed() -> Error {
        Error::new(codes::DATABASE_CONNECTION_FAILED, "Database connection failed", "database")
            .with_retryable(true)
    }

    pub fn database_query_timeout(query: &str) -> Error {
        Error::new(codes::DATABASE_QUERY_TIMEOUT, "Database query timeout", "database")
            .with_description(&format!("Query: {}", query))
            .with_retryable(true)
    }

    pub fn service_unavailable(service: &str) -> Error {
        Error::new(codes::SERVICE_UNAVAILABLE, "Service unavailable", service)
            .with_retryable(true)
    }

    pub fn connection_timeout(service: &str) -> Error {
        Error::new(codes::CONNECTION_TIMEOUT, "Connection timeout", service)
            .with_retryable(true)
    }

    pub fn invalid_request_format() -> Error {
        Error::new(codes::INVALID_REQUEST_FORMAT, "Invalid request format", "api")
    }

    pub fn missing_required_fields(fields: &[&str]) -> Error {
        Error::new(codes::MISSING_REQUIRED_FIELDS, "Missing required fields", "api")
            .with_description(&format!("Missing fields: {}", fields.join(", ")))
    }

    pub fn invalid_token_format() -> Error {
        Error::new(codes::INVALID_TOKEN_FORMAT, "Invalid token format", "auth")
    }

    pub fn token_expired() -> Error {
        Error::new(codes::TOKEN_EXPIRED, "Token expired", "auth")
    }

    pub fn insufficient_permissions(resource: &str, action: &str) -> Error {
        Error::new(codes::INSUFFICIENT_PERMISSIONS, "Insufficient permissions", "auth")
            .with_description(&format!("Resource: {}, Action: {}", resource, action))
    }

    pub fn connection_establishment_failed(peer_id: &str) -> Error {
        Error::new(codes::CONNECTION_ESTABLISHMENT_FAILED, "Connection establishment failed", "sfu")
            .with_description(&format!("Peer ID: {}", peer_id))
            .with_retryable(true)
    }

    pub fn connection_lost(peer_id: &str) -> Error {
        Error::new(codes::CONNECTION_LOST, "Connection lost", "sfu")
            .with_description(&format!("Peer ID: {}", peer_id))
            .with_retryable(true)
    }

    pub fn max_connections_reached(max: u32) -> Error {
        Error::new(codes::MAX_CONNECTIONS_REACHED, "Maximum connections reached", "sfu")
            .with_description(&format!("Maximum connections: {}", max))
    }

    pub fn storage_device_unavailable(device: &str) -> Error {
        Error::new(codes::STORAGE_DEVICE_UNAVAILABLE, "Storage device unavailable", "recording")
            .with_description(&format!("Device: {}", device))
            .with_retryable(false)
    }

    pub fn insufficient_storage_space(required: u64, available: u64) -> Error {
        Error::new(codes::INSUFFICIENT_STORAGE_SPACE, "Insufficient storage space", "recording")
            .with_description(&format!("Required: {} bytes, Available: {} bytes", required, available))
            .with_retryable(false)
    }

    pub fn recording_initialization_failed(room_id: &str) -> Error {
        Error::new(codes::RECORDING_INITIALIZATION_FAILED, "Recording initialization failed", "recording")
            .with_description(&format!("Room ID: {}", room_id))
            .with_retryable(true)
    }

    pub fn recording_encoding_failed(error: &str) -> Error {
        Error::new(codes::RECORDING_ENCODING_FAILED, "Recording encoding failed", "recording")
            .with_description(error)
            .with_retryable(false)
    }

    pub fn resource_not_found(resource_type: &str, resource_id: &str) -> Error {
        Error::new(codes::RESOURCE_NOT_FOUND, "Resource not found", "api")
            .with_description(&format!("Type: {}, ID: {}", resource_type, resource_id))
    }

    pub fn resource_already_exists(resource_type: &str, resource_id: &str) -> Error {
        Error::new(codes::RESOURCE_ALREADY_EXISTS, "Resource already exists", "api")
            .with_description(&format!("Type: {}, ID: {}", resource_type, resource_id))
    }

    pub fn operation_not_allowed(operation: &str, reason: &str) -> Error {
        Error::new(codes::OPERATION_NOT_ALLOWED, "Operation not allowed", "api")
            .with_description(&format!("Operation: {}, Reason: {}", operation, reason))
    }
}

pub struct ErrorHandler {
    service_name: String,
}

impl ErrorHandler {
    pub fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
        }
    }

    pub fn handle_error<E: StdError>(&self, error: &E, context: &str) -> Error {
        let error_message = format!("{}", error);
        let error_code = if error_message.contains("connection") {
            codes::DATABASE_CONNECTION_FAILED
        } else if error_message.contains("timeout") {
            codes::CONNECTION_TIMEOUT
        } else if error_message.contains("not found") {
            codes::RESOURCE_NOT_FOUND
        } else if error_message.contains("already exists") {
            codes::RESOURCE_ALREADY_EXISTS
        } else {
            codes::DATABASE_CONNECTION_FAILED
        };

        Error::new(error_code, &error_message, &self.service_name)
            .with_description(context)
            .with_stack_trace(&format!("{}", error))
    }

    pub fn handle_error_with_code<E: StdError>(&self, error: &E, context: &str, code: &str) -> Error {
        Error::new(code, &format!("{}", error), &self.service_name)
            .with_description(context)
            .with_stack_trace(&format!("{}", error))
    }

    pub fn handle_validation_error(&self, message: &str, field: &str) -> Error {
        Error::new(codes::INVALID_REQUEST_FORMAT, message, &self.service_name)
            .with_description(&format!("Field: {}", field))
    }

    pub fn handle_auth_error(&self, message: &str) -> Error {
        Error::new(codes::INVALID_TOKEN_FORMAT, message, &self.service_name)
    }

    pub fn handle_media_error(&self, message: &str, component: &str) -> Error {
        Error::new(codes::CONNECTION_ESTABLISHMENT_FAILED, message, &self.service_name)
            .with_description(&format!("Component: {}", component))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = Error::new("TEST-001", "Test error", "test_service");
        assert_eq!(error.code, "TEST-001");
        assert_eq!(error.message, "Test error");
        assert_eq!(error.service, "test_service");
        assert_eq!(error.severity, ErrorSeverity::Error);
        assert!(!error.retryable);
    }

    #[test]
    fn test_error_with_description() {
        let error = Error::new("TEST-002", "Test error", "test_service")
            .with_description("Test description");
        assert_eq!(error.description.unwrap(), "Test description");
    }

    #[test]
    fn test_error_with_severity() {
        let error = Error::new("TEST-003", "Test error", "test_service")
            .with_severity(ErrorSeverity::Warning);
        assert_eq!(error.severity, ErrorSeverity::Warning);
    }

    #[test]
    fn test_error_with_retryable() {
        let error = Error::new("TEST-004", "Test error", "test_service")
            .with_retryable(true);
        assert!(error.retryable);
    }

    #[test]
    fn test_error_with_details() {
        let details = serde_json::json!({ "key": "value" });
        let error = Error::new("TEST-005", "Test error", "test_service")
            .with_details(details);
        assert!(error.details.is_some());
    }

    #[test]
    fn test_error_with_metadata() {
        let error = Error::new("TEST-006", "Test error", "test_service")
            .with_metadata("test_key", serde_json::Value::String("test_value".to_string()));
        assert!(error.metadata.is_some());
        assert_eq!(error.metadata.unwrap().get("test_key").unwrap(), &serde_json::Value::String("test_value".to_string()));
    }

    #[test]
    fn test_error_to_response() {
        let error = Error::new("TEST-007", "Test error", "test_service");
        let response = error.to_response();
        assert!(response.is_object());
        assert!(response.get("error").is_some());
    }

    #[test]
    fn test_error_logging() {
        let error = Error::new("TEST-008", "Test error", "test_service");
        error.log();
        // Note: This test just verifies that logging doesn't panic
    }

    #[test]
    fn test_error_display() {
        let error = Error::new("TEST-009", "Test error", "test_service");
        assert_eq!(format!("{}", error), "TEST-009: Test error");
    }

    #[test]
    fn test_error_handlers() {
        let handler = ErrorHandler::new("test_service");
        
        let db_error = handler.handle_error(&std::io::Error::new(std::io::ErrorKind::NotFound, "test"), "test context");
        assert_eq!(db_error.code, codes::DATABASE_CONNECTION_FAILED);
        
        let validation_error = handler.handle_validation_error("Invalid field", "username");
        assert_eq!(validation_error.code, codes::INVALID_REQUEST_FORMAT);
        assert_eq!(validation_error.description.unwrap(), "Field: username");
    }
}

pub struct ErrorResponseBuilder {
    error: Error,
}

impl ErrorResponseBuilder {
    pub fn new(code: &str, message: &str, service: &str) -> Self {
        Self {
            error: Error::new(code, message, service),
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.error = self.error.with_description(description);
        self
    }

    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.error = self.error.with_severity(severity);
        self
    }

    pub fn with_retryable(mut self, retryable: bool) -> Self {
        self.error = self.error.with_retryable(retable);
        self
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.error = self.error.with_details(details);
        self
    }

    pub fn with_correlation_id(mut self, correlation_id: &str) -> Self {
        self.error = self.error.with_correlation_id(correlation_id);
        self
    }

    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.error = self.error.with_metadata(key, value);
        self
    }

    pub fn build(self) -> Error {
        self.error
    }

    pub fn build_response(&self) -> serde_json::Value {
        self.error.to_response()
    }
}

pub struct ErrorRecovery {
    max_retries: u32,
    base_delay: u64,
    max_delay: u64,
}

impl ErrorRecovery {
    pub fn new(max_retries: u32, base_delay: u64, max_delay: u64) -> Self {
        Self {
            max_retries,
            base_delay,
            max_delay,
        }
    }

    pub async fn retry_with_backoff<F, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: FnMut() -> Result<T, E>,
        E: StdError,
    {
        let mut attempt = 0;
        let mut delay = self.base_delay;

        loop {
            match operation() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempt += 1;
                    if attempt > self.max_retries {
                        return Err(e);
                    }

                    let sleep_duration = std::cmp::min(delay, self.max_delay);
                    tokio::time::sleep(std::time::Duration::from_millis(sleep_duration)).await;
                    delay *= 2;
                }
            }
        }
    }

    pub fn should_retry(&self, error: &Error) -> bool {
        error.retryable && error.severity == ErrorSeverity::Error
    }
}