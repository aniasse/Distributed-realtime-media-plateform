# DRMP - Référence API complète

## Vue d'ensemble

La plateforme DRMP expose une API RESTful via le service Gateway (port 8888) et des APIs spécifiques pour chaque service. Toutes les requêtes nécessitent une authentification JWT sauf indication contraire.

## Authentification

### Génération de token

```bash
POST /api/auth/login
Content-Type: application/json

{
  "username": "your_username",
  "password": "your_password"
}
```

### Validation de token

```bash
POST /api/auth/validate
Authorization: Bearer <token>

{
  "token": "<token>"
}
```

### En-tête d'authentification

```bash
Authorization: Bearer <jwt_token>
```

## API Gateway (port 8888)

### Health Check

```bash
GET /health
```

**Réponse :**
```json
{
  "status": "healthy",
  "services": {
    "auth": "healthy",
    "control-plane": "healthy",
    "recording": "healthy",
    "media-edge": "healthy",
    "sfu": "healthy"
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### Routage de requêtes

```bash
POST /api/{service}/{endpoint}
Authorization: Bearer <token>
Content-Type: application/json

{
  "endpoint": "rooms",
  "method": "POST",
  "body": {
    "tenant_id": "00000000-0000-0000-0000-000000000001",
    "max_participants": 10
  }
}
```

## API Control Plane (port 8080)

### Gestion des pièces

#### Créer une pièce

```bash
POST /api/rooms
Authorization: Bearer <token>
Content-Type: application/json

{
  "tenant_id": "00000000-0000-0000-0000-000000000001",
  "max_participants": 100,
  "auto_start_recording": false,
  "recording_config": {
    "storage_path": "/recordings",
    "retention_days": 30
  }
}
```

**Réponse :**
```json
{
  "room_id": "550e8400-e29b-41d4-a716-446655440000",
  "created_at": "2024-01-01T00:00:00Z",
  "max_participants": 100,
  "current_participants": 0,
  "state": "active"
}
```

#### Lister les pièces

```bash
GET /api/rooms?tenant_id=00000000-0000-0000-0000-000000000001&state=active
Authorization: Bearer <token>
```

**Réponse :**
```json
{
  "rooms": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "tenant_id": "00000000-0000-0000-0000-000000000001",
      "max_participants": 100,
      "current_participants": 5,
      "state": "active",
      "created_at": "2024-01-01T00:00:00Z",
      "recording": false
    }
  ],
  "total": 1,
  "page": 1,
  "limit": 100
}
```

#### Supprimer une pièce

```bash
DELETE /api/rooms/{room_id}
Authorization: Bearer <token>
```

**Réponse :**
```json
{
  "status": "success",
  "message": "Room deleted successfully"
}
```

### Gestion des pairs

#### Ajouter un pair

```bash
POST /api/peers
Authorization: Bearer <token>
Content-Type: application/json

{
  "room_id": "550e8400-e29b-41d4-a716-446655440000",
  "peer_id": "660e8400-e29b-41d4-a716-446655440001",
  "metadata": {
    "username": "john_doe",
    "role": "presenter",
    "device": "desktop"
  }
}
```

**Réponse :**
```json
{
  "peer_id": "660e8400-e29b-41d4-a716-446655440001",
  "room_id": "550e8400-e29b-41d4-a716-446655440000",
  "connected_at": "2024-01-01T00:00:00Z",
  "metadata": {
    "username": "john_doe",
    "role": "presenter",
    "device": "desktop"
  }
}
```

#### Mettre à jour un pair

```bash
PUT /api/peers/{peer_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "room_id": "550e8400-e29b-41d4-a716-446655440000",
  "metadata": {
    "role": "viewer",
    "status": "active"
  }
}
```

### Gestion des pistes

#### Ajouter une piste

```bash
POST /api/tracks
Authorization: Bearer <token>
Content-Type: application/json

{
  "room_id": "550e8400-e29b-41d4-a716-446655440000",
  "publisher_id": "660e8400-e29b-41d4-a716-446655440001",
  "kind": "video",
  "simulcast_layers": [
    {
      "id": "770e8400-e29b-41d4-a716-446655440002",
      "bitrate_kbps": 1000,
      "resolution": [1280, 720],
      "framerate": 30,
      "active": true
    }
  ],
  "ssrc": 1234,
  "payload_type": 96
}
```

**Réponse :**
```json
{
  "track_id": "880e8400-e29b-41d4-a716-446655440003",
  "publisher_id": "660e8400-e29b-41d4-a716-446655440001",
  "kind": "video",
  "created_at": "2024-01-01T00:00:00Z"
}
```

## API Auth Service (port 8081)

### Inscription utilisateur

```bash
POST /api/register
Content-Type: application/json

{
  "username": "john_doe",
  "email": "john@example.com",
  "password": "secure_password",
  "tenant_id": "00000000-0000-0000-0000-000000000001"
}
```

**Réponse :**
```json
{
  "user_id": "990e8400-e29b-41d4-a716-446655440004",
  "username": "john_doe",
  "email": "john@example.com",
  "created_at": "2024-01-01T00:00:00Z",
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

### Connexion

```bash
POST /api/login
Content-Type: application/json

{
  "username": "john_doe",
  "password": "secure_password"
}
```

**Réponse :**
```json
{
  "user_id": "990e8400-e29b-41d4-a716-446655440004",
  "username": "john_doe",
  "email": "john@example.com",
  "tenant_id": "00000000-0000-0000-0000-000000000001",
  "roles": ["user", "presenter"],
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_at": "2024-01-02T00:00:00Z"
}
```

### Validation de token

```bash
POST /api/validate
Authorization: Bearer <token>

{
  "token": "<token>"
}
```

**Réponse :**
```json
{
  "valid": true,
  "user": {
    "id": "990e8400-e29b-41d4-a716-446655440004",
    "username": "john_doe",
    "email": "john@example.com",
    "tenant_id": "00000000-0000-0000-0000-000000000001",
    "roles": ["user", "presenter"],
    "permissions": ["create_room", "start_recording"]
  },
  "expires_at": "2024-01-02T00:00:00Z"
}
```

### Réinitialisation de mot de passe

```bash
POST /api/reset-password
Content-Type: application/json

{
  "email": "john@example.com"
}
```

## API Recording Service (port 8080)

### Démarrer l'enregistrement

```bash
POST /api/rooms/{room_id}/record
Authorization: Bearer <token>
Content-Type: application/json

{
  "recording_config": {
    "storage_path": "/recordings",
    "retention_days": 30,
    "segment_duration": 300,
    "include_audio": true,
    "include_video": true,
    "include_data": false
  }
}
```

**Réponse :**
```json
{
  "recording_id": "aa0e8400-e29b-41d4-a716-446655440005",
  "room_id": "550e8400-e29b-41d4-a716-446655440000",
  "started_at": "2024-01-01T00:00:00Z",
  "status": "recording",
  "config": {
    "storage_path": "/recordings",
    "retention_days": 30,
    "segment_duration": 300
  }
}
```

### Arrêter l'enregistrement

```bash
DELETE /api/rooms/{room_id}/record
Authorization: Bearer <token>
```

**Réponse :**
```json
{
  "recording_id": "aa0e8400-e29b-41d4-a716-446655440005",
  "room_id": "550e8400-e29b-41d4-a716-446655440000",
  "stopped_at": "2024-01-01T00:05:00Z",
  "duration_seconds": 300,
  "file_path": "/recordings/550e8400-e29b-41d4-a716-446655440000/recording_001.mp4",
  "status": "completed"
}
```

### Lister les enregistrements

```bash
GET /api/recordings?room_id=550e8400-e29b-41d4-a716-446655440000&status=completed
Authorization: Bearer <token>
```

**Réponse :**
```json
{
  "recordings": [
    {
      "id": "aa0e8400-e29b-41d4-a716-446655440005",
      "room_id": "550e8400-e29b-41d4-a716-446655440000",
      "file_path": "/recordings/550e8400-e29b-41d4-a716-446655440000/recording_001.mp4",
      "duration_seconds": 300,
      "size_bytes": 52428800,
      "created_at": "2024-01-01T00:05:00Z",
      "status": "completed"
    }
  ],
  "total": 1,
  "page": 1,
  "limit": 100
}
```

### Télécharger un enregistrement

```bash
GET /api/recordings/{recording_id}/download
Authorization: Bearer <token>
```

### Supprimer un enregistrement

```bash
DELETE /api/recordings/{recording_id}
Authorization: Bearer <token>
```

## API Media Edge (ports 1935, 8081)

### Streaming RTMP

```bash
# Publier un flux
rtmp://localhost:1935/live/{stream_key}

# Lire un flux
rtmp://localhost:1935/live/{stream_key}
```

### Connexion WebRTC

```bash
# WebSocket pour WebRTC
ws://localhost:8081/webrtc

# Headers requis
Sec-WebSocket-Protocol: webrtc
```

### Configuration des protocoles

```bash
GET /api/protocols
Authorization: Bearer <token>
```

**Réponse :**
```json
{
  "protocols": [
    {
      "type": "webrtc",
      "enabled": true,
      "settings": {
        "port": 8081,
        "max_connections": 1000,
        "ice_servers": [
          {
            "urls": "stun:stun.l.google.com:19302"
          }
        ]
      }
    },
    {
      "type": "rtmp",
      "enabled": true,
      "settings": {
        "port": 1935,
        "max_connections": 500,
        "stream_timeout": 3600
      }
    }
  ]
}
```

## API SFU (port 5004)

### Gestion des paquets RTP

```bash
POST /api/rooms/{room_id}/rtp
Authorization: Bearer <token>
Content-Type: application/json

{
  "packet": {
    "ssrc": 1234,
    "sequence_number": 1,
    "timestamp": 1000,
    "payload_type": 96,
    "payload": "base64_encoded_payload",
    "marker": false,
    "extension": null
  },
  "peer_id": "660e8400-e29b-41d4-a716-446655440001"
}
```

### Gestion des paquets RTCP

```bash
POST /api/rooms/{room_id}/rtcp
Authorization: Bearer <token>
Content-Type: application/json

{
  "packet": {
    "packet_type": "RTPFB",
    "report_count": 1,
    "payload": "base64_encoded_payload"
  },
  "peer_id": "660e8400-e29b-41d4-a716-446655440001"
}
```

### Statistiques de la pièce

```bash
GET /api/rooms/{room_id}/stats
Authorization: Bearer <token>
```

**Réponse :**
```json
{
  "room_id": "550e8400-e29b-41d4-a716-446655440000",
  "peer_count": 5,
  "active_peers": 3,
  "track_count": 8,
  "max_participants": 100,
  "created_at": "2024-01-01T00:00:00Z",
  "packet_stats": {
    "rtp_packets_sent": 12345,
    "rtp_packets_received": 12300,
    "rtcp_packets_sent": 456,
    "rtcp_packets_received": 450,
    "packet_loss_percent": 0.36,
    "avg_latency_ms": 45.2
  }
}
```

### Configuration du SFU

```bash
GET /api/config
Authorization: Bearer <token>
```

**Réponse :**
```json
{
  "max_participants_per_room": 100,
  "max_bandwidth_mbps": 10,
  "packet_processing_timeout_ms": 100,
  "forwarding_strategies": ["unicast", "multicast", "simulcast", "svc"],
  "adaptive_bitrate": true,
  "packet_loss_recovery": true
}
```

## Modèles de données

### Room

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "tenant_id": "00000000-0000-0000-0000-000000000001",
  "max_participants": 100,
  "current_participants": 5,
  "state": "active",
  "created_at": "2024-01-01T00:00:00Z",
  "recording": false,
  "metadata": {
    "name": "Meeting Room",
    "description": "Team meeting",
    "password": "<hashed>"
  }
}
```

### Peer

```json
{
  "id": "660e8400-e29b-41d4-a716-446655440001",
  "room_id": "550e8400-e29b-41d4-a716-446655440000",
  "connected_at": "2024-01-01T00:00:00Z",
  "metadata": {
    "username": "john_doe",
    "role": "presenter",
    "device": "desktop",
    "location": "New York"
  },
  "bandwidth_estimate": {
    "available_upload": 5000,
    "available_download": 10000,
    "current_usage": 2000
  },
  "connection_state": "connected",
  "ice_candidates": [
    {
      "foundation": "1",
      "component": 1,
      "transport": "udp",
      "priority": 2130706431,
      "connection_address": "192.168.1.100",
      "port": 54321,
      "candidate_type": "host",
      "username_fragment": "abc123"
    }
  ]
}
```

### Track

```json
{
  "id": "880e8400-e29b-41d4-a716-446655440003",
  "publisher_id": "660e8400-e29b-41d4-a716-446655440001",
  "kind": "video",
  "simulcast_layers": [
    {
      "id": "770e8400-e29b-41d4-a716-446655440002",
      "bitrate_kbps": 1000,
      "resolution": [1280, 720],
      "framerate": 30,
      "active": true
    },
    {
      "id": "770e8400-e29b-41d4-a716-446655440003",
      "bitrate_kbps": 500,
      "resolution": [640, 360],
      "framerate": 15,
      "active": false
    }
  ],
  "ssrc": 1234,
  "payload_type": 96,
  "created_at": "2024-01-01T00:00:00Z"
}
```

### Recording

```json
{
  "id": "aa0e8400-e29b-41d4-a716-446655440005",
  "room_id": "550e8400-e29b-41d4-a716-446655440000",
  "file_path": "/recordings/550e8400-e29b-41d4-a716-446655440000/recording_001.mp4",
  "duration_seconds": 300,
  "size_bytes": 52428800,
  "created_at": "2024-01-01T00:05:00Z",
  "status": "completed",
  "config": {
    "include_audio": true,
    "include_video": true,
    "include_data": false,
    "segment_duration": 300
  }
}
```

## Gestion des erreurs

### Codes d'erreur

| Code | Description | HTTP Status |
|------|-------------|-------------|
| 400 | Bad Request | 400 |
| 401 | Unauthorized | 401 |
| 403 | Forbidden | 403 |
| 404 | Not Found | 404 |
| 409 | Conflict | 409 |
| 429 | Too Many Requests | 429 |
| 500 | Internal Server Error | 500 |
| 503 | Service Unavailable | 503 |

### Format d'erreur

```json
{
  "error": {
    "code": "room_not_found",
    "message": "The requested room was not found",
    "details": {
      "room_id": "550e8400-e29b-41d4-a716-446655440000"
    },
    "timestamp": "2024-01-01T00:00:00Z",
    "request_id": "bb0e8400-e29b-41d4-a716-446655440006"
  }
}
```

### Erreurs courantes

- `room_not_found`: La pièce demandée n'existe pas
- `peer_not_found`: Le pair n'existe pas
- `track_not_found`: La piste n'existe pas
- `room_full`: La pièce est pleine
- `invalid_token`: Token d'authentification invalide
- `insufficient_permissions`: Permissions insuffisantes
- `stream_key_invalid`: Clé de stream RTMP invalide
- `recording_in_progress`: Un enregistrement est déjà en cours

## WebSocket API

### Connexion

```javascript
const ws = new WebSocket('ws://localhost:8081/webrtc');
ws.onopen = () => {
  console.log('Connected to WebRTC server');
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('Received:', message);
};

ws.onclose = () => {
  console.log('Disconnected');
};
```

### Messages

#### Join Room

```json
{
  "type": "join_room",
  "room_id": "550e8400-e29b-41d4-a716-446655440000",
  "peer_id": "660e8400-e29b-41d4-a716-446655440001",
  "metadata": {
    "username": "john_doe",
    "role": "presenter"
  }
}
```

#### Leave Room

```json
{
  "type": "leave_room",
  "room_id": "550e8400-e29b-41d4-a716-446655440000",
  "peer_id": "660e8400-e29b-41d4-a716-446655440001"
}
```

#### Send RTP Packet

```json
{
  "type": "rtp",
  "packet": {
    "ssrc": 1234,
    "sequence_number": 1,
    "timestamp": 1000,
    "payload_type": 96,
    "payload": "base64_encoded_payload",
    "marker": false
  }
}
```

#### Send RTCP Packet

```json
{
  "type": "rtcp",
  "packet": {
    "packet_type": "RTPFB",
    "report_count": 1,
    "payload": "base64_encoded_payload"
  }
}
```

#### Ice Candidate

```json
{
  "type": "ice_candidate",
  "candidate": {
    "foundation": "1",
    "component": 1,
    "transport": "udp",
    "priority": 2130706431,
    "connection_address": "192.168.1.100",
    "port": 54321,
    "candidate_type": "host",
    "username_fragment": "abc123"
  }
}
```

## Rate Limiting

### Configuration

```json
{
  "rate_limit": {
    "requests_per_minute": 60,
    "burst_size": 10,
    "ip_based": true,
    "user_based": true,
    "room_based": true
  }
}
```

### En-têtes de rate limiting

```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 59
X-RateLimit-Reset: 1704067200
```

### Réponse quand limité

```json
{
  "error": {
    "code": "rate_limit_exceeded",
    "message": "Too many requests",
    "retry_after_seconds": 60
  }
}
```

## Versioning

L'API utilise le versioning via l'en-tête HTTP :

```bash
Accept: application/vnd.drmp.v1+json
```

Versions supportées :
- `v1` : Version actuelle
- `v2` : Bêta (si disponible)

## CORS

Configuration CORS pour le développement :

```bash
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS
Access-Control-Allow-Headers: Content-Type, Authorization, X-Request-ID
Access-Control-Allow-Credentials: true
```

## Documentation additionnelle

- [OpenAPI Specification](openapi.yaml)
- [WebSocket Protocol](websocket.md)
- [RTMP/WebRTC Integration](media.md)
- [Error Handling](error-handling.md)