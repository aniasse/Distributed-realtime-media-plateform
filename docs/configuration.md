# Configuration Reference

This document provides comprehensive documentation for DRMP's configuration options.

## Configuration Overview

DRMP uses a hierarchical configuration system with environment variables, configuration files, and Kubernetes ConfigMaps.

### Configuration Sources (Priority Order)

1. **Environment Variables** (Highest priority)
2. **Configuration Files** (`config.toml`, `config.json`)
3. **Default Values** (Lowest priority)

## Environment Variables

### Database Configuration

```bash
# PostgreSQL Database
DATABASE_URL=postgresql://postgres:password@localhost:5432/drmp
DATABASE_MAX_CONNECTIONS=20
DATABASE_MIN_CONNECTIONS=5
DATABASE_CONNECTION_TIMEOUT=30s
DATABASE_IDLE_TIMEOUT=10m

# Redis Cache
REDIS_URL=redis://localhost:6379
REDIS_MAX_CONNECTIONS=50
REDIS_DB=0
REDIS_PASSWORD=
```

### Media Configuration

```bash
# Room Configuration
MAX_PARTICIPANTS=100
MAX_BANDWIDTH_MBPS=10
MAX_ROOMS=1000
ROOM_EXPIRY_HOURS=24

# Video Configuration
VIDEO_CODEC=libx264
VIDEO_PRESET=fast
VIDEO_BITRATE_KBPS=2000
VIDEO_MAX_BITRATE_KBPS=4000
VIDEO_BUFFER_SIZE_KB=8000
VIDEO_GOP_SIZE=60
VIDEO_MIN_KEYFRAME_INTERVAL=30

# Audio Configuration
AUDIO_CODEC=aac
AUDIO_BITRATE_KBPS=128
AUDIO_SAMPLE_RATE=44100
AUDIO_CHANNELS=2
```

### WebRTC Configuration

```bash
# STUN/TURN Servers
WEBRTC_STUN_SERVERS=stun:stun.l.google.com:19302,stun:stun1.l.google.com:19302
WEBRTC_TURN_SERVERS=turn:turn.example.com:3478
WEBRTC_TURN_USERNAME=turn_user
WEBRTC_TURN_CREDENTIAL=turn_password

# ICE Configuration
WEBRTC_ICE_CHECK_INTERVAL=10
WEBRTC_ICE_CHECK_MAX_LOSS=95
WEBRTC_ICE_CHECK_MAX_WAIT=30
WEBRTC_ICE_TCP=false
WEBRTC_ICE_UDP=true
```

### RTMP Configuration

```bash
# RTMP Server
RTMP_PORT=1935
RTMP_CHUNK_SIZE=4096
RTMP_MAX_CONNECTIONS=1000
RTMP_PUBLISH_TIMEOUT=30s
RTMP_PLAY_TIMEOUT=30s

# Stream Authentication
RTMP_AUTH_ENABLED=true
RTMP_STREAM_KEY_VALIDITY_HOURS=24
RTMP_SHARED_SECRET=your_shared_secret
```

### Recording Configuration

```bash
# Recording Storage
RECORDING_STORAGE_PATH=/var/recordings
RECORDING_STORAGE_TYPE=local
RECORDING_RETENTION_DAYS=30
RECORDING_SEGMENT_DURATION=300
RECORDING_MAX_FILE_SIZE_MB=2048

# Recording Formats
RECORDING_FORMATS=mp4,webm,mkv
RECORDING_VIDEO_CODEC=libx264
RECORDING_AUDIO_CODEC=aac
RECORDING_CONTAINER=mp4

# Recording Quality
RECORDING_QUALITY_PRESET=medium
RECORDING_VIDEO_BITRATE_KBPS=2000
RECORDING_AUDIO_BITRATE_KBPS=128
```

### Auth Configuration

```bash
# JWT Configuration
JWT_SECRET=your-super-secret-jwt-key-change-in-production
JWT_EXPIRY_HOURS=24
JWT_REFRESH_EXPIRY_DAYS=7
JWT_ALGORITHM=HS256

# OAuth2 Configuration
OAUTH2_ENABLED=false
OAUTH2_CLIENT_ID=
OAUTH2_CLIENT_SECRET=
OAUTH2_ISSUER_URL=
OAUTH2_REDIRECT_URI=
OAUTH2_SCOPES=openid,email,profile

# LDAP Configuration
LDAP_ENABLED=false
LDAP_URL=ldap://ldap.example.com:389
LDAP_BASE_DN=dc=example,dc=com
LDAP_BIND_DN=cn=admin,dc=example,dc=com
LDAP_BIND_PASSWORD=
```

### Gateway Configuration

```bash
# Gateway Service
GATEWAY_PORT=8888
GATEWAY_HOST=0.0.0.0
GATEWAY_MAX_REQUESTS=10000
GATEWAY_REQUEST_TIMEOUT=30s
GATEWAY_KEEPALIVE_TIMEOUT=60s

# Gateway Routing
GATEWAY_ROUTES={
  "control-plane": "http://control-plane:8080",
  "sfu": "http://sfu:5004",
  "recording": "http://recording:8080",
  "auth": "http://auth:8081"
}
```

### Monitoring Configuration

```bash
# Prometheus Metrics
METRICS_PORT=9090
METRICS_PATH=/metrics
METRICS_ENABLED=true
METRICS_NAMESPACE=drmp

# Grafana Dashboards
GRAFANA_DASHBOARD_PATH=/var/grafana/dashboards
GRAFANA_DATASOURCE_URL=http://prometheus:9090

# Logging
LOG_LEVEL=info
LOG_FORMAT=json
LOG_OUTPUT=stdout
LOG_FILE_PATH=/var/log/drmp.log
LOG_MAX_SIZE_MB=100
LOG_MAX_BACKUPS=5
LOG_MAX_AGE_DAYS=30
```

### Security Configuration

```bash
# CORS Configuration
CORS_ENABLED=true
CORS_ORIGINS=http://localhost:3000,http://localhost:8080
CORS_METHODS=GET,POST,PUT,DELETE,OPTIONS
CORS_HEADERS=Content-Type,Authorization,Accept
CORS_MAX_AGE=86400

# Rate Limiting
RATE_LIMIT_ENABLED=true
RATE_LIMIT_WINDOW=1m
RATE_LIMIT_MAX_REQUESTS=100
RATE_LIMIT_BURST=10

# Security Headers
SECURITY_HEADERS_ENABLED=true
SECURITY_HEADERS_CONTENT_SECURITY_POLICY=default-src 'self'
SECURITY_HEADERS_X_FRAME_OPTIONS=SAMEORIGIN
SECURITY_HEADERS_X_CONTENT_TYPE_OPTIONS=nosniff
SECURITY_HEADERS_REFERRER_POLICY=strict-origin-when-cross-origin
```

### Kubernetes Configuration

```bash
# Namespace
K8S_NAMESPACE=drmp

# Resource Limits
K8S_CPU_REQUEST=250m
K8S_CPU_LIMIT=500m
K8S_MEMORY_REQUEST=256Mi
K8S_MEMORY_LIMIT=512Mi

# HPA Configuration
K8S_HPA_MIN_REPLICAS=2
K8S_HPA_MAX_REPLICAS=10
K8S_HPA_TARGET_CPU_UTILIZATION=70
K8S_HPA_TARGET_MEMORY_UTILIZATION=70
```

## Configuration Files

### TOML Configuration

```toml
# config.toml
[database]
url = "postgresql://postgres:password@localhost:5432/drmp"
max_connections = 20
min_connections = 5
connection_timeout = "30s"
idle_timeout = "10m"

[media]
max_participants = 100
max_bandwidth_mbps = 10
max_rooms = 1000
room_expiry_hours = 24

[video]
codec = "libx264"
preset = "fast"
bitrate_kbps = 2000
max_bitrate_kbps = 4000
buffer_size_kb = 8000
gop_size = 60
min_keyframe_interval = 30

[audio]
codec = "aac"
bitrate_kbps = 128
sample_rate = 44100
channels = 2

[webrtc]
stun_servers = ["stun:stun.l.google.com:19302", "stun:stun1.l.google.com:19302"]
turn_servers = ["turn:turn.example.com:3478"]
turn_username = "turn_user"
turn_credential = "turn_password"
ice_check_interval = 10
ice_check_max_loss = 95
ice_check_max_wait = 30
ice_tcp = false
ice_udp = true

[rtmp]
port = 1935
chunk_size = 4096
max_connections = 1000
publish_timeout = "30s"
play_timeout = "30s"
auth_enabled = true
stream_key_validity_hours = 24
shared_secret = "your_shared_secret"

[recording]
storage_path = "/var/recordings"
storage_type = "local"
retention_days = 30
segment_duration = 300
max_file_size_mb = 2048
formats = ["mp4", "webm", "mkv"]
video_codec = "libx264"
audio_codec = "aac"
container = "mp4"
quality_preset = "medium"
video_bitrate_kbps = 2000
audio_bitrate_kbps = 128

[auth]
jwt_secret = "your-super-secret-jwt-key-change-in-production"
jwt_expiry_hours = 24
jwt_refresh_expiry_days = 7
jwt_algorithm = "HS256"
oauth2_enabled = false
oauth2_client_id = ""
oauth2_client_secret = ""
oauth2_issuer_url = ""
oauth2_redirect_uri = ""
oauth2_scopes = ["openid", "email", "profile"]
ldap_enabled = false
ldap_url = "ldap://ldap.example.com:389"
ldap_base_dn = "dc=example,dc=com"
ldap_bind_dn = "cn=admin,dc=example,dc=com"
ldap_bind_password = ""

[gateway]
port = 8888
host = "0.0.0.0"
max_requests = 10000
request_timeout = "30s"
keepalive_timeout = "60s"
routes = {
  "control-plane" = "http://control-plane:8080",
  "sfu" = "http://sfu:5004",
  "recording" = "http://recording:8080",
  "auth" = "http://auth:8081"
}

[monitoring]
metrics_port = 9090
metrics_path = "/metrics"
metrics_enabled = true
metrics_namespace = "drmp"
grafana_dashboard_path = "/var/grafana/dashboards"
grafana_datasource_url = "http://prometheus:9090"

[logging]
loglevel = "info"
log_format = "json"
log_output = "stdout"
log_file_path = "/var/log/drmp.log"
log_max_size_mb = 100
log_max_backups = 5
log_max_age_days = 30

[security]
cors_enabled = true
cors_origins = ["http://localhost:3000", "http://localhost:8080"]
cors_methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
cors_headers = ["Content-Type", "Authorization", "Accept"]
cors_max_age = 86400
rate_limit_enabled = true
rate_limit_window = "1m"
rate_limit_max_requests = 100
rate_limit_burst = 10
security_headers_enabled = true
security_headers_content_security_policy = "default-src 'self'"
security_headers_x_frame_options = "SAMEORIGIN"
security_headers_x_content_type_options = "nosniff"
security_headers_referrer_policy = "strict-origin-when-cross-origin"
```

### JSON Configuration

```json
{
  "database": {
    "url": "postgresql://postgres:password@localhost:5432/drmp",
    "max_connections": 20,
    "min_connections": 5,
    "connection_timeout": "30s",
    "idle_timeout": "10m"
  },
  "media": {
    "max_participants": 100,
    "max_bandwidth_mbps": 10,
    "max_rooms": 1000,
    "room_expiry_hours": 24
  },
  "video": {
    "codec": "libx264",
    "preset": "fast",
    "bitrate_kbps": 2000,
    "max_bitrate_kbps": 4000,
    "buffer_size_kb": 8000,
    "gop_size": 60,
    "min_keyframe_interval": 30
  },
  "audio": {
    "codec": "aac",
    "bitrate_kbps": 128,
    "sample_rate": 44100,
    "channels": 2
  },
  "webrtc": {
    "stun_servers": ["stun:stun.l.google.com:19302", "stun:stun1.l.google.com:19302"],
    "turn_servers": ["turn:turn.example.com:3478"],
    "turn_username": "turn_user",
    "turn_credential": "turn_password",
    "ice_check_interval": 10,
    "ice_check_max_loss": 95,
    "ice_check_max_wait": 30,
    "ice_tcp": false,
    "ice_udp": true
  },
  "rtmp": {
    "port": 1935,
    "chunk_size": 4096,
    "max_connections": 1000,
    "publish_timeout": "30s",
    "play_timeout": "30s",
    "auth_enabled": true,
    "stream_key_validity_hours": 24,
    "shared_secret": "your_shared_secret"
  },
  "recording": {
    "storage_path": "/var/recordings",
    "storage_type": "local",
    "retention_days": 30,
    "segment_duration": 300,
    "max_file_size_mb": 2048,
    "formats": ["mp4", "webm", "mkv"],
    "video_codec": "libx264",
    "audio_codec": "aac",
    "container": "mp4",
    "quality_preset": "medium",
    "video_bitrate_kbps": 2000,
    "audio_bitrate_kbps": 128
  },
  "auth": {
    "jwt_secret": "your-super-secret-jwt-key-change-in-production",
    "jwt_expiry_hours": 24,
    "jwt_refresh_expiry_days": 7,
    "jwt_algorithm": "HS256",
    "oauth2_enabled": false,
    "oauth2_client_id": "",
    "oauth2_client_secret": "",
    "oauth2_issuer_url": "",
    "oauth2_redirect_uri": "",
    "oauth2_scopes": ["openid", "email", "profile"],
    "ldap_enabled": false,
    "ldap_url": "ldap://ldap.example.com:389",
    "ldap_base_dn": "dc=example,dc=com",
    "ldap_bind_dn": "cn=admin,dc=example,dc=com",
    "ldap_bind_password": ""
  },
  "gateway": {
    "port": 8888,
    "host": "0.0.0.0",
    "max_requests": 10000,
    "request_timeout": "30s",
    "keepalive_timeout": "60s",
    "routes": {
      "control-plane": "http://control-plane:8080",
      "sfu": "http://sfu:5004",
      "recording": "http://recording:8080",
      "auth": "http://auth:8081"
    }
  },
  "monitoring": {
    "metrics_port": 9090,
    "metrics_path": "/metrics",
    "metrics_enabled": true,
    "metrics_namespace": "drmp",
    "grafana_dashboard_path": "/var/grafana/dashboards",
    "grafana_datasource_url": "http://prometheus:9090"
  },
  "logging": {
    "loglevel": "info",
    "log_format": "json",
    "log_output": "stdout",
    "log_file_path": "/var/log/drmp.log",
    "log_max_size_mb": 100,
    "log_max_backups": 5,
    "log_max_age_days": 30
  },
  "security": {
    "cors_enabled": true,
    "cors_origins": ["http://localhost:3000", "http://localhost:8080"],
    "cors_methods": ["GET", "POST", "PUT", "DELETE", "OPTIONS"],
    "cors_headers": ["Content-Type", "Authorization", "Accept"],
    "cors_max_age": 86400,
    "rate_limit_enabled": true,
    "rate_limit_window": "1m",
    "rate_limit_max_requests": 100,
    "rate_limit_burst": 10,
    "security_headers_enabled": true,
    "security_headers_content_security_policy": "default-src 'self'",
    "security_headers_x_frame_options": "SAMEORIGIN",
    "security_headers_x_content_type_options": "nosniff",
    "security_headers_referrer_policy": "strict-origin-when-cross-origin"
  }
}
```

## Kubernetes Configuration

### ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: drmp-config
  namespace: drmp
data:
  config.toml: |
    [database]
    url = "postgresql://postgres:password@postgres:5432/drmp"
    max_connections = 20
    min_connections = 5
    connection_timeout = "30s"
    idle_timeout = "10m"
    
    [media]
    max_participants = 100
    max_bandwidth_mbps = 10
    max_rooms = 1000
    room_expiry_hours = 24
    
    [video]
    codec = "libx264"
    preset = "fast"
    bitrate_kbps = 2000
    max_bitrate_kbps = 4000
    buffer_size_kb = 8000
    gop_size = 60
    min_keyframe_interval = 30
    
    [audio]
    codec = "aac"
    bitrate_kbps = 128
    sample_rate = 44100
    channels = 2
    
    [webrtc]
    stun_servers = ["stun:stun.l.google.com:19302", "stun:stun1.l.google.com:19302"]
    turn_servers = ["turn:turn.example.com:3478"]
    turn_username = "turn_user"
    turn_credential = "turn_password"
    ice_check_interval = 10
    ice_check_max_loss = 95
    ice_check_max_wait = 30
    ice_tcp = false
    ice_udp = true
    
    [rtmp]
    port = 1935
    chunk_size = 4096
    max_connections = 1000
    publish_timeout = "30s"
    play_timeout = "30s"
    auth_enabled = true
    stream_key_validity_hours = 24
    shared_secret = "your_shared_secret"
    
    [recording]
    storage_path = "/recordings"
    storage_type = "local"
    retention_days = 30
    segment_duration = 300
    max_file_size_mb = 2048
    formats = ["mp4", "webm", "mkv"]
    video_codec = "libx264"
    audio_codec = "aac"
    container = "mp4"
    quality_preset = "medium"
    video_bitrate_kbps = 2000
    audio_bitrate_kbps = 128
    
    [auth]
    jwt_secret = "your-super-secret-jwt-key-change-in-production"
    jwt_expiry_hours = 24
    jwt_refresh_expiry_days = 7
    jwt_algorithm = "HS256"
    oauth2_enabled = false
    oauth2_client_id = ""
    oauth2_client_secret = ""
    oauth2_issuer_url = ""
    oauth2_redirect_uri = ""
    oauth2_scopes = ["openid", "email", "profile"]
    ldap_enabled = false
    ldap_url = "ldap://ldap.example.com:389"
    ldap_base_dn = "dc=example,dc=com"
    ldap_bind_dn = "cn=admin,dc=example,dc=com"
    ldap_bind_password = ""
    
    [gateway]
    port = 8888
    host = "0.0.0.0"
    max_requests = 10000
    request_timeout = "30s"
    keepalive_timeout = "60s"
    routes = {
      "control-plane" = "http://control-plane:8080",
      "sfu" = "http://sfu:5004",
      "recording" = "http://recording:8080",
      "auth" = "http://auth:8081"
    }
    
    [monitoring]
    metrics_port = 9090
    metrics_path = "/metrics"
    metrics_enabled = true
    metrics_namespace = "drmp"
    grafana_dashboard_path = "/grafana/dashboards"
    grafana_datasource_url = "http://prometheus:9090"
    
    [logging]
    loglevel = "info"
    log_format = "json"
    log_output = "stdout"
    log_file_path = "/var/log/drmp.log"
    log_max_size_mb = 100
    log_max_backups = 5
    log_max_age_days = 30
    
    [security]
    cors_enabled = true
    cors_origins = ["http://localhost:3000", "http://localhost:8080"]
    cors_methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
    cors_headers = ["Content-Type", "Authorization", "Accept"]
    cors_max_age = 86400
    rate_limit_enabled = true
    rate_limit_window = "1m"
    rate_limit_max_requests = 100
    rate_limit_burst = 10
    security_headers_enabled = true
    security_headers_content_security_policy = "default-src 'self'"
    security_headers_x_frame_options = "SAMEORIGIN"
    security_headers_x_content_type_options = "nosniff"
    security_headers_referrer_policy = "strict-origin-when-cross-origin"

  # Additional ConfigMaps for specific services
spec:
  volumes:
  - name: config-volume
    configMap:
      name: drmp-config
  containers:
  - name: drmp-gateway
    volumeMounts:
    - name: config-volume
      mountPath: /app/config
```

### Service-Specific Configuration

#### SFU Service

```toml
[sfu]
packet_processing_threads = 4
forwarding_strategy = "unicast"
max_packet_size = 1500
rtcp_report_interval = 5000
packet_loss_threshold = 0.05
latency_threshold_ms = 100

[simulcast]
enabled = true
max_layers = 3
quality_adaptation = true
buffer_based_adaptation = true
```

#### Media Edge Service

```toml
[media-edge]
max_connections = 5000
connection_timeout = "30s"
keepalive_timeout = "60s"
max_packet_size = 1500

[webrtc]
dtls_timeout = "15s"
srtp_timeout = "10s"
ice_candidate_timeout = "30s"

[rtmp]
publish_timeout = "30s"
play_timeout = "30s"
max_stream_duration = "2h"
```

#### Recording Service

```toml
[recording]
storage_path = "/recordings"
storage_type = "local"
retention_days = 30
segment_duration = 300
max_file_size_mb = 2048

[storage]
local.enabled = true
local.path = "/recordings"

[s3]
enabled = false
bucket = "drmp-recordings"
region = "us-east-1"
access_key = ""
secret_key = ""

[azure]
enabled = false
container = "recordings"
connection_string = ""
```

## Configuration Validation

### Validation Rules

```bash
# Validate configuration
cargo run -- validate-config --config config.toml

# Check for common issues
cargo run -- check-config --config config.toml

# Validate environment variables
cargo run -- validate-env --env-file .env
```

### Validation Examples

```toml
# Valid configuration example
[database]
url = "postgresql://postgres:password@localhost:5432/drmp"
max_connections = 20

# Invalid configuration example
[database]
url = "postgresql://postgres@localhost:5432/drmp"  # Missing password
max_connections = "twenty"  # Should be integer
```

### Error Messages

```
ERROR: Invalid database URL format
ERROR: max_connections must be a positive integer
ERROR: JWT secret must be at least 32 characters
ERROR: Recording storage path must exist and be writable
```

## Best Practices

### Security

```bash
# Use strong secrets
JWT_SECRET=$(openssl rand -base64 32)

# Rotate secrets regularly
# Set up secret rotation in your CI/CD pipeline

# Use environment variables for secrets
# Never commit secrets to version control
```

### Performance

```bash
# Optimize database connections
DATABASE_MAX_CONNECTIONS=20
DATABASE_MIN_CONNECTIONS=5

# Tune media settings based on your infrastructure
MAX_BANDWIDTH_MBPS=10
MAX_PARTICIPANTS=100

# Enable caching where appropriate
REDIS_ENABLED=true
REDIS_MAX_CONNECTIONS=50
```

### Scalability

```bash
# Configure for horizontal scaling
K8S_HPA_MIN_REPLICAS=2
K8S_HPA_MAX_REPLICAS=10

# Tune resource limits
K8S_CPU_REQUEST=250m
K8S_CPU_LIMIT=500m
K8S_MEMORY_REQUEST=256Mi
K8S_MEMORY_LIMIT=512Mi
```

### Monitoring

```bash
# Enable comprehensive monitoring
METRICS_ENABLED=true
LOG_LEVEL=info

# Configure alerting
ALERT_MANAGER_ENABLED=true
ALERT_MANAGER_URL=http://alertmanager:9093
```

## Troubleshooting

### Common Issues

```bash
# Database connection issues
# Check DATABASE_URL format
# Verify PostgreSQL is running
# Check network connectivity

# Media streaming issues
# Check WebRTC TURN/STUN configuration
# Verify port forwarding
# Check firewall settings

# Performance issues
# Monitor resource usage
# Check configuration limits
# Review logs for errors
```

### Debug Configuration

```bash
# Enable debug logging
RUST_LOG=debug
LOG_LEVEL=debug

# Validate configuration
cargo run -- validate-config --config config.toml

# Check environment variables
printenv | grep DRMP_
```

---

**DRMP Configuration** - Comprehensive configuration options for optimal performance and scalability.