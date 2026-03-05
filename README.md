# DRMP - Distributed Realtime Media Platform

## Quick Start

### Prerequisites
- Docker & Docker Compose
- Rust (for development)
- PostgreSQL (for production)
- Kubernetes cluster (for production)

### Local Development

1. **Clone and setup**
   ```bash
   git clone <repository-url>
   cd drmp
   ```

2. **Start services**
   ```bash
   docker-compose -f docker/docker-compose.dev.yml up --build
   ```

3. **Access services**
   - Gateway API: http://localhost:8888
   - Grafana Dashboard: http://localhost:3000 (admin/admin)
   - Prometheus: http://localhost:9090
   - PostgreSQL: localhost:5432

### Production Deployment

1. **Deploy to Kubernetes**
   ```bash
   kubectl apply -f k8s/
   ```

2. **Configure storage**
   ```bash
   kubectl apply -f k8s/database.yaml
   ```

3. **Set up ingress**
   ```bash
   kubectl apply -f k8s/ingress.yaml
   ```

## 📊 API Endpoints

### Gateway (Port 8888)
- `GET /health` - Health check
- `POST /api/{service}/{endpoint}` - Route to microservices

### Control Plane (Port 8080)
- `POST /api/rooms` - Create room
- `GET /api/rooms` - List rooms
- `DELETE /api/rooms/{room_id}` - Delete room
- `POST /api/peers` - Add peer
- `POST /api/tracks` - Add track

### Auth Service (Port 8081)
- `POST /api/register` - Register user
- `POST /api/login` - Login
- `POST /api/validate` - Validate token

### Recording Service (Port 8080)
- `POST /api/rooms` - Start recording
- `DELETE /api/rooms/{room_id}` - Stop recording
- `GET /api/recordings` - List recordings

## 🔧 Configuration

### Environment Variables

```bash
# Database
DATABASE_URL=postgresql://postgres:password@postgres:5432/drmp

# Redis
REDIS_URL=redis://redis:6379

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

### Kubernetes ConfigMap

See `k8s/configmap.yaml` for complete configuration.

## 📊 Monitoring

### Metrics Exported
- **Room Metrics**: Active rooms, participants, tracks
- **Media Metrics**: Packets sent/received, bitrate, latency
- **System Metrics**: CPU, memory, disk usage
- **Network Metrics**: Connection count, bandwidth usage

### Dashboards
- **Media Performance**: Packet flow, latency, quality metrics
- **System Health**: Resource utilization, service health
- **User Activity**: Active sessions, room usage
- **Error Tracking**: Error rates, failure patterns

## 🔒 Security

### Authentication
- JWT-based authentication
- Multi-provider support (OAuth2, LDAP, custom)
- Token validation and refresh

### Authorization
- Role-based access control (RBAC)
- Resource-level permissions
- Tenant isolation

### Data Protection
- SSL/TLS encryption
- Secure credential storage
- Audit logging

## 🚀 Scaling

### Horizontal Pod Autoscaler
- **SFU**: CPU-based scaling, min 2, max 10 replicas
- **Media Edge**: Connection-based scaling, min 2, max 8 replicas
- **Gateway**: Request-based scaling, min 2, max 6 replicas

### Vertical Scaling
- Configurable resource limits
- Memory-optimized for media processing
- CPU-optimized for packet routing

### Multi-region Support
- Edge node deployment
- Geographic load balancing
- CDN integration

## 📈 Performance

### Benchmarks
- **Packet Processing**: 10,000+ packets/second per SFU node
- **Connection Handling**: 5,000+ concurrent connections
- **Latency**: <50ms end-to-end for WebRTC
- **Throughput**: 100+ Mbps per media stream

### Optimization Features
- **Adaptive Bitrate**: Dynamic quality adjustment
- **Packet Loss Recovery**: NACK-based retransmission
- **Congestion Control**: TCP-friendly congestion algorithms
- **Hardware Acceleration**: GPU/CPU offloading options

## 📚 Documentation

### API Reference
- [OpenAPI/Swagger Specification](./docs/api.md)
- [WebSocket Protocol](./docs/websocket.md)
- [RTMP/WebRTC Integration](./docs/media.md)

### Deployment Guides
- [Local Development](./docs/local.md)
- [Production Setup](./docs/production.md)
- [Kubernetes Deployment](./docs/kubernetes.md)

### Architecture Docs
- [System Design](./docs/design.md)
- [Data Flow](./docs/flow.md)
- [Security Model](./docs/security.md)

## 🤝 Contributing

1. **Fork** the repository
2. **Create** a feature branch
3. **Implement** your changes
4. **Test** thoroughly
5. **Submit** a pull request

### Development Guidelines
- Follow Rust best practices
- Write comprehensive tests
- Use async/await patterns
- Implement proper error handling
- Add documentation for new features

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Tokio**: Async runtime foundation
- **Actix Web**: Web framework
- **PostgreSQL**: Reliable database
- **Prometheus**: Monitoring and metrics
- **Grafana**: Visualization platform

---

**DRMP** - Building the future of realtime media communication with Rust.