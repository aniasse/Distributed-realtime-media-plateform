# Getting Started Guide

This guide will help you get started with DRMP quickly, from installation to your first application.

## Prerequisites

### System Requirements

- **Operating System**: Linux, macOS, or Windows (WSL2 recommended)
- **RAM**: 4GB minimum, 8GB recommended
- **Storage**: 2GB free space
- **Network**: Stable internet connection

### Software Requirements

- **Docker**: 20.10+
- **Docker Compose**: 2.0+
- **Rust**: 1.70+ (for development)
- **Git**: 2.30+

### Development Tools

- **IDE**: VS Code, IntelliJ IDEA, or similar
- **Terminal**: Terminal with bash/zsh support
- **Package Manager**: Cargo (included with Rust)

## Installation

### Quick Installation (Recommended)

1. **Install Docker**
   - Linux: `curl -fsSL https://get.docker.com -o get-docker.sh && sh get-docker.sh`
   - macOS: Download from Docker Desktop
   - Windows: Download from Docker Desktop

2. **Install Docker Compose**
   - Linux: `sudo apt install docker-compose` or `brew install docker-compose`
   - macOS/Windows: Included with Docker Desktop

3. **Install Rust**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

4. **Clone the Repository**
   ```bash
   git clone <repository-url>
   cd drmp
   ```

### Manual Installation

For advanced users who want to build from source:

1. **Install PostgreSQL**
   ```bash
   # Ubuntu/Debian
   sudo apt update && sudo apt install postgresql postgresql-contrib
   
   # macOS
   brew install postgresql
   
   # Start PostgreSQL
   sudo systemctl start postgresql  # Linux
   brew services start postgresql    # macOS
   ```

2. **Create Database**
   ```bash
   sudo -u postgres psql
   CREATE DATABASE drmp;
   CREATE USER drmp WITH PASSWORD ''password'';
   GRANT ALL PRIVILEGES ON DATABASE drmp TO drmp;
   \q
   ```

## First Application

Let's create a simple video chat application to demonstrate DRMP's capabilities.

### Step 1: Start DRMP Services

```bash
# From the drmp directory
cd docker
docker-compose -f docker-compose.dev.yml up --build
```

This will start all services:
- Gateway (port 8888)
- Control Plane (port 8080)
- Auth Service (port 8081)
- Recording Service (port 8080)
- Media Edge (ports 1935, 8081)
- SFU (port 5004)

### Step 2: Create Your First Room

```bash
# Create a room via Gateway API
curl -X POST http://localhost:8888/api/control-plane/rooms \
  -H "Content-Type: application/json" \
  -d '{"max_participants": 10, "tenant_id": "default"}'
```

### Step 3: Join the Room

```bash
# Join the room
curl -X POST http://localhost:8888/api/control-plane/peers \
  -H "Content-Type: application/json" \
  -d '{"room_id": "<room_id>", "peer_id": "user1"}'
```

### Step 4: Add Video Track

```bash
# Add video track
curl -X POST http://localhost:8888/api/control-plane/tracks \
  -H "Content-Type: application/json" \
  -d '{"room_id": "<room_id>", "track_id": "video1", "kind": "video"}'
```

### Step 5: Start Streaming

You can now use WebRTC to connect to the room:

```javascript
// Simple WebRTC client example
const peerConnection = new RTCPeerConnection({
  iceServers: [
    { urls: 'stun:stun.l.google.com:19302' }
  ]
});

// Connect to Media Edge
const ws = new WebSocket('ws://localhost:8081');

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  if (message.type === 'offer') {
    peerConnection.setRemoteDescription(message);
    peerConnection.createAnswer().then(answer => {
      peerConnection.setLocalDescription(answer);
      ws.send(JSON.stringify(answer));
    });
  }
};

// Add video track
const videoTrack = await navigator.mediaDevices.getUserMedia({ video: true });
videoTrack.getTracks().forEach(track => peerConnection.addTrack(track));
```

## Configuration

### Environment Variables

Create a `.env` file in your project root:

```bash
# Database
DATABASE_URL=postgresql://postgres:password@localhost:5432/drmp

# Redis
REDIS_URL=redis://localhost:6379

# Media
MAX_PARTICIPANTS=100
MAX_BANDWIDTH_MBPS=10

# Recording
RECORDING_STORAGE_PATH=/recordings
RECORDING_RETENTION_DAYS=30

# Auth
JWT_SECRET=your-super-secret-jwt-key-change-in-production
TOKEN_EXPIRY_HOURS=24

# Gateway
GATEWAY_PORT=8080
GATEWAY_HOST=0.0.0.0

# Logging
RUST_LOG=info
LOG_LEVEL=info

# Monitoring
METRICS_PORT=9090
PROMETHEUS_SCRAPE_INTERVAL=30s
```

### Configuration Options

#### Media Configuration

```bash
# Maximum participants per room
MAX_PARTICIPANTS=100

# Maximum bandwidth per room (Mbps)
MAX_BANDWIDTH_MBPS=10

# Video quality presets
VIDEO_QUALITY_LOW=360p
VIDEO_QUALITY_MEDIUM=720p
VIDEO_QUALITY_HIGH=1080p
```

#### Recording Configuration

```bash
# Storage path for recordings
RECORDING_STORAGE_PATH=/var/recordings

# Retention period for recordings (days)
RECORDING_RETENTION_DAYS=30

# Segment duration for recordings (seconds)
RECORDING_SEGMENT_DURATION=300
```

## Development Workflow

### Building the Project

```bash
# Build all services
cargo build --release

# Build specific service
cargo build --package sfu --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific service
cargo test --package sfu

# Run tests with coverage
cargo tarpaulin --out Html
```

### Code Quality

```bash
# Run linter
cargo clippy -- -D warnings

# Run formatter
cargo fmt

# Run type checking
cargo check
```

### Debugging

```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Run specific service with debug logging
RUST_LOG=debug cargo run --package sfu

# Attach debugger
rust-gdb target/debug/drmp-sfu
```

## Common Issues

### Port Conflicts

If you encounter port conflicts, modify the port numbers in the configuration:

```bash
# Change Gateway port
GATEWAY_PORT=8889

# Change Media Edge ports
RTMP_PORT=1936
WEBRTC_PORT=8082
```

### Database Connection Issues

```bash
# Check PostgreSQL status
sudo systemctl status postgresql

# Check connection
psql -h localhost -p 5432 -U postgres -d drmp

# Reset database
DROP DATABASE drmp;
CREATE DATABASE drmp;
```

### Performance Issues

```bash
# Monitor resource usage
htop

# Check logs
docker-compose logs -f

# Scale services
docker-compose up --scale sfu=3
```

## Next Steps

1. **Explore the API**: Check out the [API Reference](api.md) for detailed endpoint documentation.
2. **Try Examples**: Look at the [Examples](examples/) directory for practical implementations.
3. **Deploy to Production**: Follow the [Deployment Guide](deployment.md) for production setup.
4. **Join the Community**: Connect with other developers on [Discord](https://discord.gg/your-discord).

## Support

If you encounter issues:

1. Check the [Troubleshooting Guide](troubleshooting/)
2. Search existing [GitHub Issues](https://github.com/your-repo/issues)
3. Ask questions on [Discord](https://discord.gg/your-discord)
4. Open a new issue on GitHub

---

**Ready to build amazing realtime media applications with DRMP!**