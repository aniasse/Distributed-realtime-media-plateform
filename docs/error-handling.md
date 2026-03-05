# Error Codes and Hierarchy

## Overview
This document defines the comprehensive error handling system for DRMP, designed to provide consistent error responses across all services with proper error codes, structured logging, and recovery mechanisms.

## Error Code Format

```
DRMP-{SERVICE}-{CATEGORY}-{SUBCATEGORY}-{SEVERITY}-{NUMBER}
```

- **SERVICE**: `GEN` (General), `AUTH` (Auth), `SFU` (SFU), `REC` (Recording), `MED` (Media Edge), `CP` (Control Plane), `GW` (Gateway)
- **CATEGORY**: `SYS` (System), `VAL` (Validation), `AUTH` (Authentication), `AUTHZ` (Authorization), `NET` (Network), `DB` (Database), `MEDIA` (Media Processing)
- **SUBCATEGORY**: Specific domain (e.g., `CONN` for connection, `TOKEN` for token, `ROOM` for room management)
- **SEVERITY**: `E` (Error), `W` (Warning), `I` (Info)
- **NUMBER**: Unique identifier (000-999)

## Error Hierarchy

### 1. System Errors (DRMP-GEN-SYS-E-*)

#### Database Errors
- `DRMP-GEN-SYS-E-DB-001`: Database connection failed
- `DRMP-GEN-SYS-E-DB-002`: Database query timeout
- `DRMP-GEN-SYS-E-DB-003`: Database constraint violation
- `DRMP-GEN-SYS-E-DB-004`: Database migration failed

#### Network Errors
- `DRMP-GEN-SYS-E-NET-001`: Service unavailable
- `DRMP-GEN-SYS-E-NET-002`: Connection timeout
- `DRMP-GEN-SYS-E-NET-003`: DNS resolution failed
- `DRMP-GEN-SYS-E-NET-004`: SSL/TLS handshake failed

#### Resource Errors
- `DRMP-GEN-SYS-E-RES-001`: Memory allocation failed
- `DRMP-GEN-SYS-E-RES-002`: Disk space exhausted
- `DRMP-GEN-SYS-E-RES-003`: File descriptor limit reached
- `DRMP-GEN-SYS-E-RES-004`: CPU throttling detected

### 2. Validation Errors (DRMP-GEN-VAL-E-*)

#### Request Validation
- `DRMP-GEN-VAL-E-REQ-001`: Invalid request format
- `DRMP-GEN-VAL-E-REQ-002`: Missing required fields
- `DRMP-GEN-VAL-E-REQ-003`: Invalid data type
- `DRMP-GEN-VAL-E-REQ-004`: Field length exceeded

#### Business Logic
- `DRMP-GEN-VAL-E-BUS-001`: Invalid business state
- `DRMP-GEN-VAL-E-BUS-002`: Operation not allowed in current state
- `DRMP-GEN-VAL-E-BUS-003`: Resource already exists
- `DRMP-GEN-VAL-E-BUS-004`: Resource not found

### 3. Authentication Errors (DRMP-AUTH-AUTH-E-*)

#### Token Errors
- `DRMP-AUTH-AUTH-E-TOKEN-001`: Invalid token format
- `DRMP-AUTH-AUTH-E-TOKEN-002`: Token expired
- `DRMP-AUTH-AUTH-E-TOKEN-003`: Token signature invalid
- `DRMP-AUTH-AUTH-E-TOKEN-004`: Token revoked

#### Credential Errors
- `DRMP-AUTH-AUTH-E-CRED-001`: Invalid username/password
- `DRMP-AUTH-AUTH-E-CRED-002`: Account locked
- `DRMP-AUTH-AUTH-E-CRED-003`: Account disabled
- `DRMP-AUTH-AUTH-E-CRED-004`: Too many failed attempts

### 4. Authorization Errors (DRMP-AUTH-AUTHZ-E-*)

#### Permission Errors
- `DRMP-AUTH-AUTHZ-E-PERM-001`: Insufficient permissions
- `DRMP-AUTH-AUTHZ-E-PERM-002`: Resource access denied
- `DRMP-AUTH-AUTHZ-E-PERM-003`: Operation not permitted
- `DRMP-AUTH-AUTHZ-E-PERM-004`: Tenant isolation violation

### 5. Media Processing Errors (DRMP-MEDIA-MEDIA-E-*)

#### SFU Errors
- `DRMP-SFU-MEDIA-E-CONN-001`: Connection establishment failed
- `DRMP-SFU-MEDIA-E-CONN-002`: Connection lost
- `DRMP-SFU-MEDIA-E-CONN-003`: Maximum connections reached
- `DRMP-SFU-MEDIA-E-CONN-004`: Connection timeout

#### Codec Errors
- `DRMP-SFU-MEDIA-E-CODEC-001`: Unsupported codec
- `DRMP-SFU-MEDIA-E-CODEC-002`: Codec negotiation failed
- `DRMP-SFU-MEDIA-E-CODEC-003`: Codec configuration error
- `DRMP-SFU-MEDIA-E-CODEC-004`: Codec initialization failed

#### Bandwidth Errors
- `DRMP-SFU-MEDIA-E-BW-001`: Bandwidth limit exceeded
- `DRMP-SFU-MEDIA-E-BW-002`: Adaptive bitrate failure
- `DRMP-SFU-MEDIA-E-BW-003`: Congestion control failure
- `DRMP-SFU-MEDIA-E-BW-004`: Packet loss recovery failed

### 6. Recording Errors (DRMP-REC-MEDIA-E-*)

#### Storage Errors
- `DRMP-REC-MEDIA-E-STORE-001`: Storage device unavailable
- `DRMP-REC-MEDIA-E-STORE-002`: Insufficient storage space
- `DRMP-REC-MEDIA-E-STORE-003`: File write error
- `DRMP-REC-MEDIA-E-STORE-004`: File corruption detected

#### Processing Errors
- `DRMP-REC-MEDIA-E-PROC-001`: Recording initialization failed
- `DRMP-REC-MEDIA-E-PROC-002`: Recording encoding failed
- `DRMP-REC-MEDIA-E-PROC-003`: Recording muxing failed
- `DRMP-REC-MEDIA-E-PROC-004`: Recording metadata generation failed

## Error Response Structure

### API Response Format

```json
{
  "error": {
    "code": "DRMP-SFU-MEDIA-E-CONN-001",
    "message": "Connection establishment failed",
    "description": "Failed to establish WebRTC connection with peer ID abc123",
    "timestamp": "2024-01-15T10:30:45Z",
    "service": "sfu",
    "severity": "error",
    "retryable": true,
    "details": {
      "peer_id": "abc123",
      "attempt": 3,
      "max_attempts": 5
    }
  }
}
```

### WebSocket Error Format

```json
{
  "type": "error",
  "payload": {
    "code": "DRMP-AUTH-AUTHZ-E-PERM-001",
    "message": "Insufficient permissions",
    "timestamp": "2024-01-15T10:30:45Z",
    "retryable": false
  }
}
```

## Error Logging Structure

### Structured Log Format

```json
{
  "timestamp": "2024-01-15T10:30:45Z",
  "level": "error",
  "service": "sfu",
  "component": "connection_manager",
  "error_code": "DRMP-SFU-MEDIA-E-CONN-001",
  "message": "Connection establishment failed",
  "correlation_id": "req-abc123",
  "request_id": "req-xyz789",
  "user_id": "user-456",
  "peer_id": "abc123",
  "attempt": 3,
  "max_attempts": 5,
  "stack_trace": "...",
  "metadata": {
    "ip_address": "192.168.1.100",
    "user_agent": "WebRTC/1.0",
    "room_id": "room-789"
  }
}
```

## Error Recovery Mechanisms

### Automatic Retry Strategy

```rust
enum RetryStrategy {
    Immediate,
    ExponentialBackoff {
        base_delay: Duration,
        max_delay: Duration,
        max_attempts: u32,
    },
    FixedDelay {
        delay: Duration,
        max_attempts: u32,
    },
    NoRetry,
}

struct RetryPolicy {
    strategy: RetryStrategy,
    should_retry: fn(&Error) -> bool,
    on_failure: fn(&Error) -> RecoveryAction,
}

enum RecoveryAction {
    LogAndContinue,
    FallbackToAlternative,
    NotifyUser,
    Escalate,
}
```

### Circuit Breaker Pattern

```rust
struct CircuitBreaker {
    state: CircuitState,
    failure_threshold: u32,
    recovery_timeout: Duration,
    monitoring_window: Duration,
}

enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}
```

## Error Monitoring and Alerting

### Key Metrics

1. **Error Rate**: Errors per second by type
2. **Error Distribution**: Errors by service and component
3. **Mean Time to Recovery (MTTR)**: Time to resolve errors
4. **Error Impact**: User impact and affected resources
5. **Error Trends**: Error patterns over time

### Alerting Rules

```yaml
# High error rate alert
- alert: HighErrorRate
  expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.1
  for: 2m
  labels:
    severity: critical
  annotations:
    summary: "High error rate detected"
    description: "Error rate is {{ $value }} errors/second"

# Database connection errors
- alert: DatabaseConnectionErrors
  expr: increase(database_connection_errors_total[5m]) > 5
  for: 1m
  labels:
    severity: warning
  annotations:
    summary: "Database connection errors detected"
    description: "{{ $value }} database connection errors in last 5 minutes"
```

## Implementation Guidelines

### 1. Error Wrapping

```rust
// Wrap errors with context
let result = some_operation().map_err(|e| {
    Error::wrap(e, "Failed to process request")
        .with_code("DRMP-GEN-SYS-E-NET-002")
        .with_metadata("request_id", request_id)
        .with_metadata("user_id", user_id)
});
```

### 2. Error Propagation

```rust
// Propagate errors with proper context
async fn process_request(request: Request) -> Result<Response, Error> {
    let validated = validate_request(request).await.map_err(|e| {
        e.with_context("Request validation failed")
            .with_code("DRMP-GEN-VAL-E-REQ-001")
    })?;
    
    let authorized = authorize_request(validated).await.map_err(|e| {
        e.with_context("Authorization failed")
            .with_code("DRMP-AUTH-AUTHZ-E-PERM-001")
    })?;
    
    let result = process_business_logic(authorized).await.map_err(|e| {
        e.with_context("Business logic processing failed")
            .with_code("DRMP-GEN-VAL-E-BUS-001")
    })?;
    
    Ok(result)
}
```

### 3. Error Recovery

```rust
// Implement error recovery
async fn resilient_operation() -> Result<(), Error> {
    let mut attempts = 0;
    let max_attempts = 3;
    let delay = Duration::from_millis(100);
    
    loop {
        match critical_operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempts += 1;
                if attempts >= max_attempts {
                    return Err(e);
                }
                
                // Apply retry strategy
                tokio::time::sleep(delay).await;
            }
        }
    }
}
```

## Testing Error Scenarios

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_database_connection_error() {
        let error = Error::database_connection_failed();
        assert_eq!(error.code(), "DRMP-GEN-SYS-E-DB-001");
        assert_eq!(error.severity(), "error");
        assert!(error.retryable());
    }
    
    #[tokio::test]
    async fn test_validation_error() {
        let error = Error::validation("Invalid request format", "Missing required field 'username'");
        assert_eq!(error.code(), "DRMP-GEN-VAL-E-REQ-001");
        assert_eq!(error.severity(), "error");
        assert!(!error.retryable());
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use httptest::*;
    
    #[tokio::test]
    async fn test_error_response_format() {
        let server = Server::run()
            .await
            .respond_with(Response::builder()
                .status(500)
                .body("{ \"error\": { \"code\": \"DRMP-GEN-SYS-E-NET-001\", \"message\": \"Service unavailable\" } }"));
        
        let client = Client::new();
        let response = client.get(server.url("/test")).await;
        
        assert!(response.status().is_server_error());
        assert!(response.json::<ErrorResponse>().await.is_ok());
    }
}
```

## Best Practices

1. **Consistent Error Codes**: Use the defined error code format consistently
2. **Structured Logging**: Include all relevant context in error logs
3. **Graceful Degradation**: Implement fallback mechanisms for non-critical errors
4. **User-Friendly Messages**: Provide clear, actionable error messages to users
5. **Security Considerations**: Never expose sensitive information in error responses
6. **Monitoring Integration**: Ensure all errors are properly tracked and alerted on
7. **Documentation**: Maintain up-to-date error documentation for developers