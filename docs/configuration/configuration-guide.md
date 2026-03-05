# DRMP - Configuration détaillée

## Vue d'ensemble de la configuration

DRMP utilise une approche multi-niveaux pour la configuration :

1. **Variables d'environnement** : Configuration principale
2. **Fichiers de configuration** : Configurations spécifiques
3. **Kubernetes ConfigMap** : Configuration pour le déploiement
4. **Runtime configuration** : Configuration dynamique

## Variables d'environnement

### Base de données

```bash
# PostgreSQL
DATABASE_URL=postgresql://username:password@localhost:5432/drmp
DATABASE_POOL_SIZE=10
DATABASE_MAX_CONNECTIONS=50
DATABASE_TIMEOUT=30

# Redis
REDIS_URL=redis://localhost:6379
REDIS_DB=0
REDIS_PASSWORD=your_redis_password
REDIS_POOL_SIZE=5
```

### Authentification

```bash
# JWT
JWT_SECRET=your-super-secret-jwt-key-change-in-production
JWT_EXPIRY_HOURS=24
JWT_ISSUER=drmp
JWT_AUDIENCE=drmp-api

# Multi-provider auth
AUTH_PROVIDERS=local,oauth2,ldap
LOCAL_AUTH_ENABLED=true
OAUTH2_ENABLED=false
LDAP_ENABLED=false

# OAuth2 providers
OAUTH2_GITHUB_CLIENT_ID=your_client_id
OAUTH2_GITHUB_CLIENT_SECRET=your_client_secret
OAUTH2_GITHUB_CALLBACK_URL=http://localhost:8888/api/auth/callback

# LDAP
LDAP_SERVER=ldap://localhost:389
LDAP_BINDDN=cn=admin,dc=example,dc=com
LDAP_BINDPASSWORD=your_password
LDAP_BASEDN=dc=example,dc=com
```

### Média

```bash
# Configuration générale
MAX_PARTICIPANTS=100
MAX_BANDWIDTH_MBPS=10
MAX_STREAM_BITRATE_KBPS=5000
MAX_VIDEO_RESOLUTION=1920x1080
MAX_FRAME_RATE=60

# WebRTC
WEBRTC_PORT=8081
WEBRTC_MAX_CONNECTIONS=1000
WEBRTC_ICE_SERVERS=[{"urls": "stun:stun.l.google.com:19302"}]
WEBRTC_CERTIFICATE_PATH=/etc/ssl/certs/webrtc.crt
WEBRTC_PRIVATE_KEY_PATH=/etc/ssl/private/webrtc.key

# RTMP
RTMP_PORT=1935
RTMP_MAX_CONNECTIONS=500
RTMP_STREAM_TIMEOUT=3600
RTMP_AUTH_REQUIRED=false

# SFU
SFU_PORT=5004
SFU_MAX_ROOMS=1000
SFU_MAX_TRACKS_PER_ROOM=50
SFU_PACKET_PROCESSING_TIMEOUT=100
SFU_FORWARDING_STRATEGY=adaptive

# Simulcast
SIMULCAST_ENABLED=true
SIMULCAST_LAYERS=[{"bitrate": 1000, "resolution": [1280, 720]}, {"bitrate": 500, "resolution": [640, 360]}]

# SVC (Scalable Video Coding)
SVC_ENABLED=false
SVC_LAYERS=3
```

### Enregistrement

```bash
# Configuration de l'enregistrement
RECORDING_ENABLED=true
RECORDING_STORAGE_PATH=/recordings
RECORDING_RETENTION_DAYS=30
RECORDING_SEGMENT_DURATION=300
RECORDING_INCLUDE_AUDIO=true
RECORDING_INCLUDE_VIDEO=true
RECORDING_INCLUDE_DATA=false

# Storage backends
RECORDING_STORAGE_BACKEND=local
s3_enabled=false
gcs_enabled=false
azure_enabled=false

# S3 configuration
S3_BUCKET=your-bucket
S3_REGION=us-east-1
S3_ACCESS_KEY=your_access_key
S3_SECRET_KEY=your_secret_key
S3_ENDPOINT=your_endpoint

# Google Cloud Storage
GCS_BUCKET=your-bucket
gcs_credentials_path=/path/to/credentials.json

# Azure Blob Storage
AZURE_CONNECTION_STRING=your_connection_string
AZURE_CONTAINER=your-container
```

### Monitoring

```bash
# Prometheus
PROMETHEUS_ENABLED=true
PROMETHEUS_PORT=9090
PROMETHEUS_SCRAPE_INTERVAL=30s
PROMETHEUS_METRICS_PATH=/metrics

# Grafana
GRAFANA_ENABLED=true
GRAFANA_URL=http://localhost:3000
GRAFANA_USERNAME=admin
GRAFANA_PASSWORD=admin

# Logging
LOG_LEVEL=info
LOG_FORMAT=json
LOG_OUTPUT=stdout
LOG_FILE_PATH=/var/log/drmp/drmp.log
LOG_MAX_SIZE=100MB
LOG_MAX_BACKUPS=5
LOG_MAX_AGE=30

# Tracing
TRACING_ENABLED=false
TRACING_PROVIDER=jaeger
tracing_endpoint=http://localhost:14268/api/traces
```

### Sécurité

```bash
# SSL/TLS
SSL_ENABLED=true
SSL_CERTIFICATE_PATH=/etc/ssl/certs/drmp.crt
SSL_PRIVATE_KEY_PATH=/etc/ssl/private/drmp.key
SSL_DHPARAM_PATH=/etc/ssl/certs/dhparam.pem

# CORS
CORS_ALLOWED_ORIGINS=*
CORS_ALLOWED_METHODS=GET,POST,PUT,DELETE,OPTIONS
CORS_ALLOWED_HEADERS=Content-Type,Authorization,X-Request-ID
CORS_ALLOW_CREDENTIALS=true

# Rate limiting
RATE_LIMIT_ENABLED=true
RATE_LIMIT_REQUESTS_PER_MINUTE=60
RATE_LIMIT_BURST_SIZE=10
RATE_LIMIT_IP_BASED=true
RATE_LIMIT_USER_BASED=true
RATE_LIMIT_ROOM_BASED=true

# Content Security Policy
CSP_ENABLED=true
CSP_DEFAULT_SRC=self
CSP_SCRIPT_SRC=self
CSP_STYLE_SRC=self
CSP_IMG_SRC=self data:
```

### Scaling et performance

```bash
# Horizontal Pod Autoscaler
HPA_ENABLED=true
HPA_MIN_REPLICAS=2
HPA_MAX_REPLICAS=10
HPA_TARGET_CPU_UTILIZATION=70
HPA_TARGET_MEMORY_UTILIZATION=80

# Vertical scaling
RESOURCE_REQUESTS_CPU=500m
RESOURCE_REQUESTS_MEMORY=512Mi
RESOURCE_LIMITS_CPU=2
RESOURCE_LIMITS_MEMORY=2Gi

# Connection pooling
CONNECTION_POOL_SIZE=20
CONNECTION_MAX_LIFETIME=300
CONNECTION_IDLE_TIMEOUT=60

# Cache configuration
CACHE_ENABLED=true
CACHE_TTL=300
CACHE_MAX_SIZE=1000
```

### Configuration spécifique aux services

#### Gateway

```bash
# Gateway configuration
GATEWAY_PORT=8888
GATEWAY_HOST=0.0.0.0
GATEWAY_TIMEOUT=30
GATEWAY_MAX_REQUEST_SIZE=10MB
GATEWAY_ENABLE_CORS=true
GATEWAY_ENABLE_RATE_LIMITING=true
GATEWAY_ENABLE_METRICS=true
```

#### Control Plane

```bash
# Control Plane configuration
CONTROL_PLANE_PORT=8080
CONTROL_PLANE_MAX_ROOMS=1000
CONTROL_PLANE_ROOM_CLEANUP_INTERVAL=300
CONTROL_PLANE_PEER_TIMEOUT=300
CONTROL_PLANE_TRACK_CLEANUP_INTERVAL=60
```

#### Auth Service

```bash
# Auth Service configuration
AUTH_SERVICE_PORT=8081
AUTH_SERVICE_PASSWORD_MIN_LENGTH=8
AUTH_SERVICE_PASSWORD_REQUIRE_UPPER=true
AUTH_SERVICE_PASSWORD_REQUIRE_LOWER=true
AUTH_SERVICE_PASSWORD_REQUIRE_NUMBER=true
AUTH_SERVICE_PASSWORD_REQUIRE_SPECIAL=true
```

#### Recording Service

```bash
# Recording Service configuration
RECORDING_SERVICE_PORT=8082
RECORDING_SERVICE_MAX_CONCURRENT_RECORDINGS=10
RECORDING_SERVICE_SEGMENT_SIZE=300
RECORDING_SERVICE_STORAGE_CHECK_INTERVAL=60
```

#### Media Edge

```bash
# Media Edge configuration
MEDIA_EDGE_PORT=1935
MEDIA_EDGE_WEBRTC_PORT=8081
MEDIA_EDGE_MAX_RTMP_CONNECTIONS=500
MEDIA_EDGE_MAX_WEBRTC_CONNECTIONS=1000
MEDIA_EDGE_CONNECTION_TIMEOUT=30
```

#### SFU

```bash
# SFU configuration
SFU_PORT=5004
SFU_MAX_PACKET_SIZE=1500
SFU_MAX_QUEUE_SIZE=1000
SFU_WORKER_THREADS=4
SFU_FORWARDING_STRATEGY=adaptive
```

## Fichiers de configuration

### Configuration JSON

```json
{
  "database": {
    "url": "postgresql://drmp:drmp123@localhost:5432/drmp",
    "pool_size": 10,
    "max_connections": 50,
    "timeout": 30
  },
  "auth": {
    "jwt": {
      "secret": "your-super-secret-jwt-key-change-in-production",
      "expiry_hours": 24,
      "issuer": "drmp",
      "audience": "drmp-api"
    },
    "providers": ["local", "oauth2", "ldap"],
    "local": {
      "enabled": true
    },
    "oauth2": {
      "enabled": false,
      "providers": {
        "github": {
          "client_id": "your_client_id",
          "client_secret": "your_client_secret",
          "callback_url": "http://localhost:8888/api/auth/callback"
        }
      }
    },
    "ldap": {
      "enabled": false,
      "server": "ldap://localhost:389",
      "bind_dn": "cn=admin,dc=example,dc=com",
      "bind_password": "your_password",
      "base_dn": "dc=example,dc=com"
    }
  },
  "media": {
    "max_participants": 100,
    "max_bandwidth_mbps": 10,
    "webrtc": {
      "port": 8081,
      "max_connections": 1000,
      "ice_servers": [
        {
          "urls": "stun:stun.l.google.com:19302"
        }
      ]
    },
    "rtmp": {
      "port": 1935,
      "max_connections": 500,
      "stream_timeout": 3600
    },
    "sfu": {
      "port": 5004,
      "max_rooms": 1000,
      "max_tracks_per_room": 50,
      "packet_processing_timeout": 100,
      "forwarding_strategy": "adaptive"
    }
  },
  "recording": {
    "enabled": true,
    "storage_path": "/recordings",
    "retention_days": 30,
    "segment_duration": 300,
    "include_audio": true,
    "include_video": true,
    "include_data": false,
    "storage_backend": "local",
    "s3": {
      "enabled": false,
      "bucket": "your-bucket",
      "region": "us-east-1",
      "access_key": "your_access_key",
      "secret_key": "your_secret_key",
      "endpoint": "your_endpoint"
    },
    "gcs": {
      "enabled": false,
      "bucket": "your-bucket",
      "credentials_path": "/path/to/credentials.json"
    },
    "azure": {
      "enabled": false,
      "connection_string": "your_connection_string",
      "container": "your-container"
    }
  },
  "monitoring": {
    "prometheus": {
      "enabled": true,
      "port": 9090,
      "scrape_interval": "30s",
      "metrics_path": "/metrics"
    },
    "grafana": {
      "enabled": true,
      "url": "http://localhost:3000",
      "username": "admin",
      "password": "admin"
    },
    "logging": {
      "level": "info",
      "format": "json",
      "output": "stdout",
      "file_path": "/var/log/drmp/drmp.log",
      "max_size": "100MB",
      "max_backups": 5,
      "max_age": 30
    },
    "tracing": {
      "enabled": false,
      "provider": "jaeger",
      "endpoint": "http://localhost:14268/api/traces"
    }
  },
  "security": {
    "ssl": {
      "enabled": true,
      "certificate_path": "/etc/ssl/certs/drmp.crt",
      "private_key_path": "/etc/ssl/private/drmp.key",
      "dhparam_path": "/etc/ssl/certs/dhparam.pem"
    },
    "cors": {
      "allowed_origins": "*",
      "allowed_methods": "GET,POST,PUT,DELETE,OPTIONS",
      "allowed_headers": "Content-Type,Authorization,X-Request-ID",
      "allow_credentials": true
    },
    "rate_limiting": {
      "enabled": true,
      "requests_per_minute": 60,
      "burst_size": 10,
      "ip_based": true,
      "user_based": true,
      "room_based": true
    }
  },
  "scaling": {
    "hpa": {
      "enabled": true,
      "min_replicas": 2,
      "max_replicas": 10,
      "target_cpu_utilization": 70,
      "target_memory_utilization": 80
    },
    "resources": {
      "requests": {
        "cpu": "500m",
        "memory": "512Mi"
      },
      "limits": {
        "cpu": "2",
        "memory": "2Gi"
      }
    },
    "connection_pooling": {
      "size": 20,
      "max_lifetime": 300,
      "idle_timeout": 60
    },
    "cache": {
      "enabled": true,
      "ttl": 300,
      "max_size": 1000
    }
  },
  "services": {
    "gateway": {
      "port": 8888,
      "host": "0.0.0.0",
      "timeout": 30,
      "max_request_size": "10MB",
      "enable_cors": true,
      "enable_rate_limiting": true,
      "enable_metrics": true
    },
    "control_plane": {
      "port": 8080,
      "max_rooms": 1000,
      "room_cleanup_interval": 300,
      "peer_timeout": 300,
      "track_cleanup_interval": 60
    },
    "auth_service": {
      "port": 8081,
      "password_min_length": 8,
      "password_require_upper": true,
      "password_require_lower": true,
      "password_require_number": true,
      "password_require_special": true
    },
    "recording_service": {
      "port": 8082,
      "max_concurrent_recordings": 10,
      "segment_size": 300,
      "storage_check_interval": 60
    },
    "media_edge": {
      "port": 1935,
      "webrtc_port": 8081,
      "max_rtmp_connections": 500,
      "max_webrtc_connections": 1000,
      "connection_timeout": 30
    },
    "sfu": {
      "port": 5004,
      "max_packet_size": 1500,
      "max_queue_size": 1000,
      "worker_threads": 4,
      "forwarding_strategy": "adaptive"
    }
  }
}
```

### Configuration YAML

```yaml
database:
  url: postgresql://drmp:drmp123@localhost:5432/drmp
  pool_size: 10
  max_connections: 50
  timeout: 30

auth:
  jwt:
    secret: your-super-secret-jwt-key-change-in-production
    expiry_hours: 24
    issuer: drmp
    audience: drmp-api
  providers: [local, oauth2, ldap]
  local:
    enabled: true
  oauth2:
    enabled: false
    providers:
      github:
        client_id: your_client_id
        client_secret: your_client_secret
        callback_url: http://localhost:8888/api/auth/callback
  ldap:
    enabled: false
    server: ldap://localhost:389
    bind_dn: cn=admin,dc=example,dc=com
    bind_password: your_password
    base_dn: dc=example,dc=com

media:
  max_participants: 100
  max_bandwidth_mbps: 10
  webrtc:
    port: 8081
    max_connections: 1000
    ice_servers:
      - urls: stun:stun.l.google.com:19302
  rtmp:
    port: 1935
    max_connections: 500
    stream_timeout: 3600
  sfu:
    port: 5004
    max_rooms: 1000
    max_tracks_per_room: 50
    packet_processing_timeout: 100
    forwarding_strategy: adaptive

recording:
  enabled: true
  storage_path: /recordings
  retention_days: 30
  segment_duration: 300
  include_audio: true
  include_video: true
  include_data: false
  storage_backend: local
  s3:
    enabled: false
    bucket: your-bucket
    region: us-east-1
    access_key: your_access_key
    secret_key: your_secret_key
    endpoint: your_endpoint
  gcs:
    enabled: false
    bucket: your-bucket
    credentials_path: /path/to/credentials.json
  azure:
    enabled: false
    connection_string: your_connection_string
    container: your-container

monitoring:
  prometheus:
    enabled: true
    port: 9090
    scrape_interval: 30s
    metrics_path: /metrics
  grafana:
    enabled: true
    url: http://localhost:3000
    username: admin
    password: admin
  logging:
    level: info
    format: json
    output: stdout
    file_path: /var/log/drmp/drmp.log
    max_size: 100MB
    max_backups: 5
    max_age: 30
  tracing:
    enabled: false
    provider: jaeger
    endpoint: http://localhost:14268/api/traces

security:
  ssl:
    enabled: true
    certificate_path: /etc/ssl/certs/drmp.crt
    private_key_path: /etc/ssl/private/drmp.key
    dhparam_path: /etc/ssl/certs/dhparam.pem
  cors:
    allowed_origins: '*'
    allowed_methods: GET,POST,PUT,DELETE,OPTIONS
    allowed_headers: Content-Type,Authorization,X-Request-ID
    allow_credentials: true
  rate_limiting:
    enabled: true
    requests_per_minute: 60
    burst_size: 10
    ip_based: true
    user_based: true
    room_based: true

scaling:
  hpa:
    enabled: true
    min_replicas: 2
    max_replicas: 10
    target_cpu_utilization: 70
    target_memory_utilization: 80
  resources:
    requests:
      cpu: 500m
      memory: 512Mi
    limits:
      cpu: 2
      memory: 2Gi
  connection_pooling:
    size: 20
    max_lifetime: 300
    idle_timeout: 60
  cache:
    enabled: true
    ttl: 300
    max_size: 1000

services:
  gateway:
    port: 8888
    host: 0.0.0.0
    timeout: 30
    max_request_size: 10MB
    enable_cors: true
    enable_rate_limiting: true
    enable_metrics: true
  control_plane:
    port: 8080
    max_rooms: 1000
    room_cleanup_interval: 300
    peer_timeout: 300
    track_cleanup_interval: 60
  auth_service:
    port: 8081
    password_min_length: 8
    password_require_upper: true
    password_require_lower: true
    password_require_number: true
    password_require_special: true
  recording_service:
    port: 8082
    max_concurrent_recordings: 10
    segment_size: 300
    storage_check_interval: 60
  media_edge:
    port: 1935
    webrtc_port: 8081
    max_rtmp_connections: 500
    max_webrtc_connections: 1000
    connection_timeout: 30
  sfu:
    port: 5004
    max_packet_size: 1500
    max_queue_size: 1000
    worker_threads: 4
    forwarding_strategy: adaptive
```

## Configuration Kubernetes

### ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: drmp-config
  namespace: drmp
data:
  # Base de données
database-url: postgresql://drmp:drmp123@localhost:5432/drmp
database-pool-size: "10"

  # Authentification
  jwt-secret: your-super-secret-jwt-key-change-in-production
  jwt-expiry-hours: "24"
  
  # Média
  max-participants: "100"
  max-bandwidth-mbps: "10"
  webrtc-port: "8081"
  rtmp-port: "1935"
  sfu-port: "5004"
  
  # Enregistrement
  recording-enabled: "true"
  recording-storage-path: "/recordings"
  recording-retention-days: "30"
  
  # Monitoring
  prometheus-enabled: "true"
  prometheus-port: "9090"
  grafana-url: "http://localhost:3000"
  
  # Sécurité
  ssl-enabled: "true"
  cors-allowed-origins: "*"
  rate-limit-enabled: "true"
  
  # Scaling
  hpa-enabled: "true"
  hpa-min-replicas: "2"
  hpa-max-replicas: "10"
```

### Secrets

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: drmp-secrets
  namespace: drmp
type: Opaque
data:
  # Base de données
  database-password: ZHJtcDEyMzQ=  # drmp1234
  
  # Authentification
  jwt-secret: eW91ci1zdXBlci1zZWNyZXQta2V5
  
  # Stockage
  s3-access-key: eW91cl9hY2Nlc3Nfa2V5
  s3-secret-key: eW91cl9zZWNyZXRfa2V5
  
  # SSL
  ssl-certificate: LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0t
  ssl-private-key: LS0tLS1CRUdJTiBQUklWQVRFIEtFWS0tLS0t
```

## Configuration runtime

### Mise à jour dynamique

```rust
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub max_participants: u32,
    pub max_bandwidth_mbps: u32,
    pub ice_servers: Vec<IceServer>,
    pub recording_enabled: bool,
    pub rate_limit_requests_per_minute: u32,
}

#[derive(Debug, Clone)]
pub struct IceServer {
    pub urls: Vec<String>,
    pub username: Option<String>,
    pub credential: Option<String>,
}

impl RuntimeConfig {
    pub fn new() -> Self {
        Self {
            max_participants: 100,
            max_bandwidth_mbps: 10,
            ice_servers: vec![
                IceServer {
                    urls: vec!["stun:stun.l.google.com:19302".to_string()],
                    username: None,
                    credential: None,
                }
            ],
            recording_enabled: true,
            rate_limit_requests_per_minute: 60,
        }
    }
    
    pub fn update_max_participants(&mut self, value: u32) {
        self.max_participants = value;
    }
    
    pub fn update_ice_servers(&mut self, servers: Vec<IceServer>) {
        self.ice_servers = servers;
    }
}
```

### Watch de configuration

```rust
use tokio::sync::watch;

pub struct ConfigWatcher {
    config_rx: watch::Receiver<RuntimeConfig>,
    config_tx: watch::Sender<RuntimeConfig>,
}

impl ConfigWatcher {
    pub fn new(initial_config: RuntimeConfig) -> Self {
        let (tx, rx) = watch::channel(initial_config);
        Self { config_rx: rx, config_tx: tx }
    }
    
    pub fn subscribe(&self) -> watch::Receiver<RuntimeConfig> {
        self.config_rx.clone()
    }
    
    pub fn update(&mut self, new_config: RuntimeConfig) {
        let _ = self.config_tx.send(new_config);
    }
}
```

## Validation de configuration

### Validation des variables d'environnement

```rust
use std::env;
use std::num::ParseIntError;

#[derive(Debug)]
pub enum ConfigError {
    Missing(String),
    Invalid(String, String),
    ParseInt(ParseIntError),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Missing(key) => write!(f, "Missing required environment variable: {}", key),
            ConfigError::Invalid(key, value) => write!(f, "Invalid value for {}: {}", key, value),
            ConfigError::ParseInt(err) => write!(f, "Parse error: {}", err),
        }
    }
}

fn validate_database_url() -> Result<(), ConfigError> {
    let url = env::var("DATABASE_URL").map_err(|_| ConfigError::Missing("DATABASE_URL".to_string()))?;
    if url.is_empty() {
        return Err(ConfigError::Invalid("DATABASE_URL".to_string(), "empty".to_string()));
    }
    Ok(())
}

fn validate_port(key: &str) -> Result<(), ConfigError> {
    let port_str = env::var(key).map_err(|_| ConfigError::Missing(key.to_string()))?;
    let port: u16 = port_str.parse().map_err(|e| ConfigError::ParseInt(e))?;
    if port == 0 || port > 65535 {
        return Err(ConfigError::Invalid(key.to_string(), port_str));
    }
    Ok(())
}
```

### Validation de configuration JSON/YAML

```rust
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub media: MediaConfig,
    pub recording: RecordingConfig,
    pub monitoring: MonitoringConfig,
    pub security: SecurityConfig,
    pub scaling: ScalingConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
    pub max_connections: u32,
    pub timeout: u32,
}

impl AppConfig {
    pub fn load_from_file(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path).map_err(|e| ConfigError::Invalid("file".to_string(), e.to_string()))?;
        
        let config: AppConfig = serde_json::from_str(&content)
            .or_else(|_| serde_yaml::from_str(&content))
            .map_err(|e| ConfigError::Invalid("config".to_string(), e.to_string()))?;
        
        config.validate()
    }
    
    fn validate(&self) -> Result<(), ConfigError> {
        // Validation de la base de données
        if self.database.url.is_empty() {
            return Err(ConfigError::Invalid("database.url".to_string(), "empty".to_string()));
        }
        
        // Validation des ports
        if self.media.webrtc.port == 0 || self.media.webrtc.port > 65535 {
            return Err(ConfigError::Invalid("media.webrtc.port".to_string(), self.media.webrtc.port.to_string()));
        }
        
        // Validation des participants
        if self.media.max_participants == 0 {
            return Err(ConfigError::Invalid("media.max_participants".to_string(), "must be > 0".to_string()));
        }
        
        Ok(())
    }
}
```

## Bonnes pratiques de configuration

### Sécurité

1. **Ne jamais commiter les secrets** : Utiliser des secrets Kubernetes ou des variables d'environnement
2. **Rotation des clés** : Mettre en place une rotation automatique des clés JWT
3. **Validation d'entrée** : Toujours valider les configurations externes
4. **Least privilege** : Donner les permissions minimales nécessaires

### Performance

1. **Pooling** : Configurer correctement les pools de connexions
2. **Caching** : Activer le cache pour les données statiques
3. **Timeouts** : Configurer des timeouts appropriés
4. **Monitoring** : Surveiller les métriques de configuration

### Fiabilité

1. **Fallback** : Avoir des configurations de fallback
2. **Validation** : Valider les configurations au démarrage
3. **Hot reload** : Supporter la mise à jour de configuration sans interruption
4. **Backup** : Sauvegarder les configurations importantes

### Déploiement

1. **Environnement** : Avoir des configurations différentes par environnement
2. **Versioning** : Versionner les fichiers de configuration
3. **Documentation** : Documenter toutes les options de configuration
4. **Testing** : Tester les configurations dans un environnement de pré-production