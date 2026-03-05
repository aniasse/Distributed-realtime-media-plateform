# API Reference

This document provides comprehensive documentation for DRMP's REST API and WebSocket protocols.

## API Overview

DRMP provides a unified API gateway that routes requests to appropriate microservices. All API requests go through the Gateway service on port 8888.

### Base URL

```
http://localhost:8888/api/{service}/{endpoint}
```

### Authentication

All API endpoints require JWT authentication except for public endpoints.

```http
Authorization: Bearer <jwt_token>
```

## REST API Endpoints

### Gateway Service (Port 8888)

#### Health Check

```http
GET /health
```

**Response**
```json
{
  "status": "healthy",
  "services": {
    "control-plane": "healthy",
    "sfu": "healthy",
    "recording": "healthy",
    "auth": "healthy"
  }
}
```

### Control Plane Service (Port 8080)

#### Create Room

```http
POST /api/control-plane/rooms
```

**Request Body**
```json
{
  "name": "Meeting Room",
  "max_participants": 10,
  "tenant_id": "default",
  "recording_enabled": true,
  "quality_preset": "medium"
}
```

**Response**
```json
{
  "room_id": "uuid",
  "created_at": "2024-01-01T00:00:00Z",
  "status": "created"
}
```

#### List Rooms

```http
GET /api/control-plane/rooms
```

**Response**
```json
{
  "rooms": [
    {
      "room_id": "uuid",
      "name": "Meeting Room",
      "participant_count": 5,
      "max_participants": 10,
      "status": "active",
      "created_at": "2024-01-01T00:00:00Z"
    }
  ]
}
```

#### Get Room Details

```http
GET /api/control-plane/rooms/{room_id}
```

**Response**
```json
{
  "room_id": "uuid",
  "name": "Meeting Room",
  "max_participants": 10,
  "participant_count": 5,
  "tracks": [
    {
      "track_id": "uuid",
      "kind": "video",
      "publisher_id": "uuid",
      "ssrc": 1234
    }
  ],
  "created_at": "2024-01-01T00:00:00Z",
  "status": "active"
}
```

#### Delete Room

```http
DELETE /api/control-plane/rooms/{room_id}
```

**Response**
```json
{
  "status": "deleted",
  "room_id": "uuid"
}
```

#### Add Peer

```http
POST /api/control-plane/peers
```

**Request Body**
```json
{
  "room_id": "uuid",
  "peer_id": "user1",
  "display_name": "John Doe",
  "role": "participant"
}
```

**Response**
```json
{
  "peer_id": "user1",
  "room_id": "uuid",
  "joined_at": "2024-01-01T00:00:00Z",
  "status": "joined"
}
```

#### Remove Peer

```http
DELETE /api/control-plane/peers/{room_id}/{peer_id}
```

**Response**
```json
{
  "peer_id": "user1",
  "room_id": "uuid",
  "left_at": "2024-01-01T00:00:00Z",
  "status": "left"
}
```

#### Add Track

```http
POST /api/control-plane/tracks
```

**Request Body**
```json
{
  "room_id": "uuid",
  "track_id": "video1",
  "kind": "video",
  "publisher_id": "user1",
  "ssrc": 1234,
  "payload_type": 96,
  "quality_preset": "medium"
}
```

**Response**
```json
{
  "track_id": "video1",
  "room_id": "uuid",
  "added_at": "2024-01-01T00:00:00Z",
  "status": "added"
}
```

#### Remove Track

```http
DELETE /api/control-plane/tracks/{room_id}/{track_id}
```

**Response**
```json
{
  "track_id": "video1",
  "room_id": "uuid",
  "removed_at": "2024-01-01T00:00:00Z",
  "status": "removed"
}
```

### Auth Service (Port 8081)

#### Register User

```http
POST /api/auth/register
```

**Request Body**
```json
{
  "username": "john.doe",
  "email": "john@example.com",
  "password": "password123",
  "tenant_id": "default"
}
```

**Response**
```json
{
  "user_id": "uuid",
  "username": "john.doe",
  "email": "john@example.com",
  "created_at": "2024-01-01T00:00:00Z",
  "status": "registered"
}
```

#### Login

```http
POST /api/auth/login
```

**Request Body**
```json
{
  "username": "john.doe",
  "password": "password123"
}
```

**Response**
```json
{
  "token": "jwt_token",
  "user": {
    "user_id": "uuid",
    "username": "john.doe",
    "email": "john@example.com",
    "roles": ["user", "tenant_admin"],
    "tenant_id": "default"
  }
}
```

#### Validate Token

```http
POST /api/auth/validate
```

**Request Body**
```json
{
  "token": "jwt_token"
}
```

**Response**
```json
{
  "valid": true,
  "user": {
    "user_id": "uuid",
    "username": "john.doe",
    "email": "john@example.com",
    "roles": ["user", "tenant_admin"],
    "tenant_id": "default"
  }
}
```

#### Get User Profile

```http
GET /api/auth/profile/{user_id}
```

**Response**
```json
{
  "user_id": "uuid",
  "username": "john.doe",
  "email": "john@example.com",
  "display_name": "John Doe",
  "roles": ["user", "tenant_admin"],
  "tenant_id": "default",
  "created_at": "2024-01-01T00:00:00Z"
}
```

#### Update Profile

```http
PUT /api/auth/profile/{user_id}
```

**Request Body**
```json
{
  "display_name": "John Doe",
  "avatar_url": "https://example.com/avatar.jpg"
}
```

**Response**
```json
{
  "user_id": "uuid",
  "username": "john.doe",
  "email": "john@example.com",
  "display_name": "John Doe",
  "avatar_url": "https://example.com/avatar.jpg",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

### Recording Service (Port 8080)

#### Start Recording

```http
POST /api/recording/rooms
```

**Request Body**
```json
{
  "room_id": "uuid",
  "recording_name": "Meeting Recording",
  "recording_format": "mp4",
  "quality_preset": "high"
}
```

**Response**
```json
{
  "recording_id": "uuid",
  "room_id": "uuid",
  "started_at": "2024-01-01T00:00:00Z",
  "status": "recording"
}
```

#### Stop Recording

```http
DELETE /api/recording/rooms/{room_id}
```

**Response**
```json
{
  "recording_id": "uuid",
  "room_id": "uuid",
  "stopped_at": "2024-01-01T00:00:00Z",
  "duration_seconds": 3600,
  "file_size_mb": 500,
  "status": "completed"
}
```

#### List Recordings

```http
GET /api/recording/recordings
```

**Query Parameters**
- `room_id`: Filter by room
- `user_id`: Filter by user
- `date_from`: Filter by date range
- `date_to`: Filter by date range

**Response**
```json
{
  "recordings": [
    {
      "recording_id": "uuid",
      "room_id": "uuid",
      "user_id": "uuid",
      "recording_name": "Meeting Recording",
      "recording_format": "mp4",
      "quality_preset": "high",
      "duration_seconds": 3600,
      "file_size_mb": 500,
      "created_at": "2024-01-01T00:00:00Z",
      "status": "completed"
    }
  ]
}
```

#### Get Recording Details

```http
GET /api/recording/recordings/{recording_id}
```

**Response**
```json
{
  "recording_id": "uuid",
  "room_id": "uuid",
  "user_id": "uuid",
  "recording_name": "Meeting Recording",
  "recording_format": "mp4",
  "quality_preset": "high",
  "duration_seconds": 3600,
  "file_size_mb": 500,
  "created_at": "2024-01-01T00:00:00Z",
  "status": "completed",
  "download_url": "https://example.com/recordings/uuid.mp4"
}
```

#### Delete Recording

```http
DELETE /api/recording/recordings/{recording_id}
```

**Response**
```json
{
  "recording_id": "uuid",
  "deleted_at": "2024-01-01T00:00:00Z",
  "status": "deleted"
}
```

## WebSocket Protocol

### Connection Establishment

```javascript
// Connect to Media Edge WebSocket
const ws = new WebSocket('ws://localhost:8081');

// Connection events
ws.onopen = () => {
  console.log('Connected to Media Edge');
};

ws.onclose = () => {
  console.log('Disconnected from Media Edge');
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};
```

### Message Format

All WebSocket messages are JSON objects with the following structure:

```json
{
  "type": "message_type",
  "data": {},
  "timestamp": "2024-01-01T00:00:00Z",
  "id": "uuid"
}
```

### WebRTC Signaling

#### Offer

```json
{
  "type": "offer",
  "data": {
    "sdp": "v=0...",
    "type": "offer"
  }
}
```

#### Answer

```json
{
  "type": "answer",
  "data": {
    "sdp": "v=0...",
    "type": "answer"
  }
}
```

#### ICE Candidate

```json
{
  "type": "ice_candidate",
  "data": {
    "candidate": "candidate:...",
    "sdpMid": "audio",
    "sdpMLineIndex": 0
  }
}
```

### Media Control Messages

#### Start Streaming

```json
{
  "type": "start_streaming",
  "data": {
    "track_id": "video1",
    "quality_preset": "medium",
    "simulcast": true
  }
}
```

#### Stop Streaming

```json
{
  "type": "stop_streaming",
  "data": {
    "track_id": "video1"
  }
}
```

#### Change Quality

```json
{
  "type": "change_quality",
  "data": {
    "track_id": "video1",
    "quality_preset": "high"
  }
}
```

#### Mute/Unmute

```json
{
  "type": "mute",
  "data": {
    "track_id": "audio1",
    "muted": true
  }
}
```

### Room Management Messages

#### Create Room

```json
{
  "type": "create_room",
  "data": {
    "name": "Meeting Room",
    "max_participants": 10,
    "recording_enabled": true
  }
}
```

#### Join Room

```json
{
  "type": "join_room",
  "data": {
    "room_id": "uuid",
    "display_name": "John Doe"
  }
}
```

#### Leave Room

```json
{
  "type": "leave_room",
  "data": {
    "room_id": "uuid"
  }
}
```

### Event Messages

#### Room Created

```json
{
  "type": "room_created",
  "data": {
    "room_id": "uuid",
    "name": "Meeting Room",
    "max_participants": 10,
    "created_at": "2024-01-01T00:00:00Z"
  }
}
```

#### Peer Joined

```json
{
  "type": "peer_joined",
  "data": {
    "peer_id": "user1",
    "room_id": "uuid",
    "display_name": "John Doe",
    "joined_at": "2024-01-01T00:00:00Z"
  }
}
```

#### Peer Left

```json
{
  "type": "peer_left",
  "data": {
    "peer_id": "user1",
    "room_id": "uuid",
    "left_at": "2024-01-01T00:00:00Z"
  }
}
```

#### Track Added

```json
{
  "type": "track_added",
  "data": {
    "track_id": "video1",
    "room_id": "uuid",
    "kind": "video",
    "publisher_id": "user1",
    "added_at": "2024-01-01T00:00:00Z"
  }
}
```

#### Track Removed

```json
{
  "type": "track_removed",
  "data": {
    "track_id": "video1",
    "room_id": "uuid",
    "removed_at": "2024-01-01T00:00:00Z"
  }
}
```

#### Room Ended

```json
{
  "type": "room_ended",
  "data": {
    "room_id": "uuid",
    "ended_at": "2024-01-01T00:00:00Z",
    "reason": "max_participants_reached"
  }
}
```

## RTMP Protocol

### Connection

```bash
# Connect to RTMP server
rtmp://localhost:1935/live/stream_key
```

### Stream Publishing

```bash
# Publish stream using ffmpeg
ffmpeg -f avfoundation -i "1" -c:v libx264 -preset fast -b:v 2000k -maxrate 2000k -bufsize 2000k \
       -c:a aac -b:a 128k -f flv rtmp://localhost:1935/live/stream_key
```

### Stream Playback

```bash
# Play stream using ffplay
ffplay rtmp://localhost:1935/live/stream_key
```

## WebRTC Protocol

### Signaling

```javascript
// WebRTC peer connection
const peerConnection = new RTCPeerConnection({
  iceServers: [
    { urls: 'stun:stun.l.google.com:19302' },
    { urls: 'turn:turn.example.com:3478', username: 'user', credential: 'pass' }
  ]
});

// Add tracks
const videoTrack = await navigator.mediaDevices.getUserMedia({ video: true });
videoTrack.getTracks().forEach(track => peerConnection.addTrack(track));

// Create offer
const offer = await peerConnection.createOffer();
await peerConnection.setLocalDescription(offer);

// Send offer via WebSocket
ws.send(JSON.stringify(offer));
```

### Media Configuration

```javascript
// Set media constraints
const constraints = {
  video: {
    width: { ideal: 1280 },
    height: { ideal: 720 },
    frameRate: { ideal: 30 }
  },
  audio: {
    echoCancellation: true,
    noiseSuppression: true
  }
};

// Get media stream
const stream = await navigator.mediaDevices.getUserMedia(constraints);
```

## Error Handling

### HTTP Status Codes

- `200 OK`: Successful request
- `201 Created`: Resource created successfully
- `204 No Content`: Successful request with no content
- `400 Bad Request`: Invalid request parameters
- `401 Unauthorized`: Authentication required
- `403 Forbidden`: Insufficient permissions
- `404 Not Found`: Resource not found
- `409 Conflict`: Resource conflict
- `429 Too Many Requests`: Rate limit exceeded
- `500 Internal Server Error`: Server error
- `503 Service Unavailable`: Service unavailable

### Error Response Format

```json
{
  "error": {
    "code": "error_code",
    "message": "Human readable error message",
    "details": {
      "field": "error details"
    }
  }
}
```

### Common Error Codes

- `INVALID_REQUEST`: Invalid request parameters
- `AUTHENTICATION_FAILED`: Authentication failed
- `AUTHORIZATION_FAILED`: Authorization failed
- `RESOURCE_NOT_FOUND`: Resource not found
- `ROOM_FULL`: Room is full
- `STREAM_KEY_INVALID`: Invalid stream key
- `RECORDING_FAILED`: Recording failed
- `INTERNAL_ERROR`: Internal server error

## Rate Limiting

### Limits

- **Authentication**: 10 requests/minute
- **Room Management**: 20 requests/minute
- **Media Operations**: 50 requests/minute
- **Recording Operations**: 10 requests/minute

### Headers

```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 99
X-RateLimit-Reset: 1640995200
```

### Exceeding Limits

```http
429 Too Many Requests
Content-Type: application/json

{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded. Try again later."
  }
}
```

## Pagination

### Query Parameters

- `page`: Page number (default: 1)
- `limit`: Items per page (default: 20, max: 100)
- `sort`: Sort field (default: created_at)
- `order`: Sort order (asc/desc, default: desc)

### Response Format

```json
{
  "data": [...],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 100,
    "pages": 5,
    "has_next": true,
    "has_prev": false
  }
}
```

## Versioning

API versioning is handled through URL path:

```
/v1/api/{service}/{endpoint}
```

Current version: `v1`

## OpenAPI Specification

The complete OpenAPI 3.0 specification is available at:

```
http://localhost:8888/openapi.json
```

## WebSocket Events

### Event Types

- `room_created`: New room created
- `peer_joined`: Peer joined a room
- `peer_left`: Peer left a room
- `track_added`: Track added to a room
- `track_removed`: Track removed from a room
- `recording_started`: Recording started
- `recording_stopped`: Recording stopped
- `quality_changed`: Quality changed
- `mute_status_changed`: Mute status changed

### Event Format

```json
{
  "event": "event_type",
  "data": {},
  "timestamp": "2024-01-01T00:00:00Z",
  "room_id": "uuid"
}
```

## Security Considerations

### Authentication

- All API endpoints require JWT authentication
- Use HTTPS in production
- Implement proper token refresh mechanisms

### Authorization

- Role-based access control (RBAC)
- Resource-level permissions
- Tenant isolation

### Rate Limiting

- Protect against abuse
- Implement exponential backoff
- Monitor for unusual patterns

### Input Validation

- Validate all input parameters
- Sanitize user inputs
- Use parameterized queries

---

**DRMP API** - Comprehensive, scalable, and secure media platform API.