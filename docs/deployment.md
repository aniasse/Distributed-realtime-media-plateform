# Deployment Guides

This document provides comprehensive deployment guides for DRMP in various environments.

## Table of Contents

- [Single-Node Setup](#single-node-setup)
- [Multi-Node Cluster Setup](#multi-node-cluster-setup)
- [Cloud Deployment](#cloud-deployment)
- [On-Premise Deployment](#on-premise-deployment)
- [Edge Computing Deployment](#edge-computing-deployment)
- [Kubernetes Deployment](#kubernetes-deployment)
- [Docker Compose Deployment](#docker-compose-deployment)
- [Production Checklist](#production-checklist)

## Single-Node Setup

### Overview

Single-node setup is ideal for development, testing, and small-scale deployments.

### Prerequisites

- Docker & Docker Compose
- 8GB RAM minimum
- 20GB free storage
- Stable internet connection

### Installation

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd drmp
   ```

2. **Configure environment**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. **Start services**
   ```bash
   docker-compose -f docker/docker-compose.dev.yml up --build
   ```

4. **Verify installation**
   ```bash
   curl http://localhost:8888/health
   ```

### Service Configuration

#### Ports

| Service | Port | Protocol |
|---------|------|----------|
| Gateway | 8888 | HTTP |
| Control Plane | 8080 | HTTP |
| Auth Service | 8081 | HTTP |
| Recording Service | 8080 | HTTP |
| Media Edge | 1935, 8081 | RTMP/WebRTC |
| SFU | 5004 | Media |
| PostgreSQL | 5432 | Database |
| Redis | 6379 | Cache |
| Prometheus | 9090 | Monitoring |
| Grafana | 3000 | Visualization |

#### Resources

- **CPU**: 2 cores minimum
- **Memory**: 8GB minimum
- **Storage**: 20GB minimum
- **Network**: 100 Mbps minimum

### Monitoring

Access monitoring dashboards:

- **Grafana**: http://localhost:3000 (admin/admin)
- **Prometheus**: http://localhost:9090

### Scaling

For single-node scaling, use Docker Compose:

```bash
# Scale specific services
docker-compose up --scale sfu=2 --scale media-edge=2
```

## Multi-Node Cluster Setup

### Overview

Multi-node setup is ideal for production environments with high availability requirements.

### Prerequisites

- Kubernetes cluster (3+ nodes)
- 16GB RAM per node minimum
- 50GB storage per node minimum
- Load balancer
- Persistent storage

### Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│     Node 1      │    │     Node 2      │    │     Node 3      │
│  ┌─────────────┐│    │  ┌─────────────┐│    │  ┌─────────────┐│
│  │   Gateway   ││    │  │   Gateway   ││    │  │   Gateway   ││
│  │   (8888)    ││    │  │   (8888)    ││    │  │   (8888)    ││
│  └─────────────┘│    │  └─────────────┘│    │  └─────────────┘│
│                 │    │                 │    │                 │
│  ┌─────────────┐│    │  ┌─────────────┐│    │  ┌─────────────┐│
│  │   Media     ││    │  │   Media     ││    │  │   Media     ││
│  │   Edge      ││    │  │   Edge      ││    │  │   Edge      ││
│  │ (1935, 8081)││    │  │ (1935, 8081)││    │  │ (1935, 8081)││
│  └─────────────┘│    │  └─────────────┘│    │  └─────────────┘│
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Load Balancer (8888)                        │
└─────────────────────────────────────────────────────────────────┘
```

### Installation

1. **Deploy Kubernetes manifests**
   ```bash
   kubectl apply -f k8s/
   ```

2. **Configure storage**
   ```bash
   kubectl apply -f k8s/database.yaml
   kubectl apply -f k8s/redis.yaml
   ```

3. **Set up ingress**
   ```bash
   kubectl apply -f k8s/ingress.yaml
   ```

4. **Configure ConfigMap**
   ```bash
   kubectl apply -f k8s/configmap.yaml
   ```

### Service Configuration

#### Kubernetes Services

```yaml
# Gateway Service
apiVersion: v1
kind: Service
metadata:
  name: gateway
spec:
  type: LoadBalancer
  ports:
  - port: 80
    targetPort: 8080
    name: http
  selector:
    app: gateway

# Media Edge Service
apiVersion: v1
kind: Service
metadata:
  name: media-edge
spec:
  type: LoadBalancer
  ports:
  - port: 1935
    targetPort: 1935
    name: rtmp
  - port: 8081
    targetPort: 8081
    name: webrtc
  selector:
    app: media-edge
```

#### Horizontal Pod Autoscaler

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: sfu
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: sfu
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
```

### Monitoring

```bash
# Deploy monitoring stack
kubectl apply -f k8s/monitoring/

# Access dashboards
kubectl port-forward svc/grafana 3000:3000
kubectl port-forward svc/prometheus 9090:9090
```

### Scaling

```bash
# Scale services manually
kubectl scale deployment sfu --replicas=5
kubectl scale deployment media-edge --replicas=3

# Auto-scaling with HPA
kubectl autoscale deployment sfu --cpu-percent=70 --min=2 --max=10
```

## Cloud Deployment

### Overview

Cloud deployment provides scalability, reliability, and global reach.

### Supported Cloud Providers

- **AWS**
- **Google Cloud Platform**
- **Microsoft Azure**
- **DigitalOcean**

### AWS Deployment

#### Prerequisites

- AWS account
- EKS cluster
- VPC with subnets
- IAM roles

#### Installation

1. **Deploy EKS cluster**
   ```bash
   eksctl create cluster --name drmp --region us-east-1 --nodegroup-name standard-workers --node-type t3.medium --nodes 3
   ```

2. **Deploy DRMP**
   ```bash
   kubectl apply -f k8s/aws/
   ```

3. **Configure AWS services**
   ```bash
   # RDS for PostgreSQL
   # ElastiCache for Redis
   # S3 for storage
   ```

#### Service Configuration

```yaml
# AWS-specific configuration
apiVersion: v1
kind: ConfigMap
metadata:
  name: drmp-config-aws
data:
  config.toml: |
    [database]
    url = "postgresql://user:password.rds.amazonaws.com:5432/drmp"
    
    [storage]
    s3.enabled = true
    s3.bucket = "drmp-recordings"
    s3.region = "us-east-1"
    
    [monitoring]
    cloudwatch.enabled = true
    cloudwatch.namespace = "DRMP"
```

### GCP Deployment

#### Prerequisites

- GCP account
- GKE cluster
- VPC network
- IAM service accounts

#### Installation

1. **Deploy GKE cluster**
   ```bash
   gcloud container clusters create drmp --zone us-central1 --machine-type n1-standard-2 --num-nodes 3
   ```

2. **Deploy DRMP**
   ```bash
   kubectl apply -f k8s/gcp/
   ```

3. **Configure GCP services**
   ```bash
   # Cloud SQL for PostgreSQL
   # Memorystore for Redis
   # Cloud Storage for storage
   ```

#### Service Configuration

```yaml
# GCP-specific configuration
apiVersion: v1
kind: ConfigMap
metadata:
  name: drmp-config-gcp
data:
  config.toml: |
    [database]
    url = "postgresql://user:password:5432/drmp?host=/cloudsql/project:region:instance"
    
    [storage]
    gcs.enabled = true
    gcs.bucket = "drmp-recordings"
    gcs.credentials_path = "/var/secrets/google/cloudstorage.json"
    
    [monitoring]
    stackdriver.enabled = true
    stackdriver.project_id = "your-project-id"
```

### Azure Deployment

#### Prerequisites

- Azure account
- AKS cluster
- Virtual network
- Service principal

#### Installation

1. **Deploy AKS cluster**
   ```bash
   az aks create --resource-group drmp --name drmp-cluster --node-count 3 --node-vm-size Standard_D2_v2
   ```

2. **Deploy DRMP**
   ```bash
   kubectl apply -f k8s/azure/
   ```

3. **Configure Azure services**
   ```bash
   # Azure Database for PostgreSQL
   # Azure Cache for Redis
   # Azure Blob Storage
   ```

#### Service Configuration

```yaml
# Azure-specific configuration
apiVersion: v1
kind: ConfigMap
metadata:
  name: drmp-config-azure
data:
  config.toml: |
    [database]
    url = "postgresql://user:password.postgres.database.azure.com:5432/drmp"
    
    [storage]
    blob.enabled = true
    blob.account_name = "yourstorageaccount"
    blob.container = "recordings"
    blob.sas_token = "your-sas-token"
    
    [monitoring]
    application_insights.enabled = true
    application_insights.instrumentation_key = "your-instrumentation-key"
```

## On-Premise Deployment

### Overview

On-premise deployment provides full control over infrastructure and data.

### Prerequisites

- Physical servers or VMs
- Network infrastructure
- Storage systems
- Backup solutions

### Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Server 1      │    │   Server 2      │    │   Server 3      │
│  ┌─────────────┐│    │  ┌─────────────┐│    │  ┌─────────────┐│
│  │   Gateway   ││    │  │   Gateway   ││    │  │   Gateway   ││
│  │   (8888)    ││    │  │   (8888)    ││    │  │   (8888)    ││
│  └─────────────┘│    │  └─────────────┘│    │  └─────────────┘│
│                 │    │                 │    │                 │
│  ┌─────────────┐│    │  ┌─────────────┐│    │  ┌─────────────┐│
│  │   Media     ││    │  │   Media     ││    │  │   Media     ││
│  │   Edge      ││    │  │   Edge      ││    │  │   Edge      ││
│  │ (1935, 8081)││    │  │ (1935, 8081)││    │  │ (1935, 8081)││
│  └─────────────┘│    │  └─────────────┘│    │  └─────────────┘│
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Load Balancer (8888)                        │
└─────────────────────────────────────────────────────────────────┘
```

### Installation

1. **Prepare servers**
   ```bash
   # Install Docker and Kubernetes
   curl -fsSL https://get.docker.com -o get-docker.sh
   sh get-docker.sh
   
   # Install Kubernetes
   kubeadm init
   ```

2. **Deploy DRMP**
   ```bash
   kubectl apply -f k8s/on-premise/
   ```

3. **Configure storage**
   ```bash
   # Setup NFS for shared storage
   # Configure local storage for each node
   ```

### Network Configuration

```yaml
# Load balancer configuration
apiVersion: v1
kind: Service
metadata:
  name: gateway-loadbalancer
spec:
  type: LoadBalancer
  ports:
  - port: 80
    targetPort: 8080
    name: http
  selector:
    app: gateway
  loadBalancerIP: 192.168.1.100
```

### Storage Configuration

```yaml
# NFS storage for recordings
apiVersion: v1
kind: PersistentVolume
metadata:
  name: recordings-pv
spec:
  capacity:
    storage: 1000Gi
  accessModes:
    - ReadWriteMany
  nfs:
    server: nfs-server.local
    path: /recordings
```

## Edge Computing Deployment

### Overview

Edge computing deployment brings processing closer to users for reduced latency.

### Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Edge Node 1   │    │   Edge Node 2   │    │   Edge Node 3   │
│  ┌─────────────┐│    │  ┌─────────────┐│    │  ┌─────────────┐│
│  │   Gateway   ││    │  │   Gateway   ││    │  │   Gateway   ││
│  │   (8888)    ││    │  │   (8888)    ││    │  │   (8888)    ││
│  └─────────────┘│    │  └─────────────┘│    │  └─────────────┘│
│                 │    │                 │    │                 │
│  ┌─────────────┐│    │  ┌─────────────┐│    │  ┌─────────────┐│
│  │   Media     ││    │  │   Media     ││    │  │   Media     ││
│  │   Edge      ││    │  │   Edge      ││    │  │   Edge      ││
│  │ (1935, 8081)││    │  │ (1935, 8081)││    │  │ (1935, 8081)││
│  └─────────────┘│    │  └─────────────┘│    │  └─────────────┘│
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Central Control Plane                        │
└─────────────────────────────────────────────────────────────────┘
```

### Prerequisites

- Edge servers in multiple geographic locations
- Low-latency network connections
- Content delivery network (CDN)
- Distributed storage

### Installation

1. **Deploy edge nodes**
   ```bash
   # Install DRMP on each edge node
   kubectl apply -f k8s/edge/
   ```

2. **Configure central control plane**
   ```bash
   kubectl apply -f k8s/central/
   ```

3. **Set up CDN integration**
   ```bash
   # Configure CDN for media delivery
   # Set up DNS for geographic routing
   ```

### Configuration

```yaml
# Edge node configuration
apiVersion: v1
kind: ConfigMap
metadata:
  name: drmp-edge-config
data:
  config.toml: |
    [edge]
    enabled = true
    region = "us-east"
    latency_threshold_ms = 50
    
    [cdn]
    enabled = true
    provider = "cloudflare"
    cache_ttl = "3600"
    
    [storage]
    edge_storage.enabled = true
    edge_storage.path = "/var/edge/recordings"
```

## Kubernetes Deployment

### Prerequisites

- Kubernetes cluster (1.20+)
- kubectl (1.20+)
- Helm (3.0+)

### Installation

1. **Deploy with Helm**
   ```bash
   helm repo add drmp https://charts.drmp.io
   helm install drmp drmp/drmp --namespace drmp --create-namespace
   ```

2. **Deploy with manifests**
   ```bash
   kubectl apply -f k8s/manifests/
   ```

### Configuration

```yaml
# Complete Kubernetes deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: drmp-gateway
spec:
  replicas: 3
  selector:
    matchLabels:
      app: gateway
  template:
    metadata:
      labels:
        app: gateway
        tier: api
    spec:
      containers:
      - name: gateway
        image: drmp/gateway:latest
        ports:
        - containerPort: 8080
          name: http
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: drmp-secrets
              key: database-url
        resources:
          requests:
            cpu: "250m"
            memory: "256Mi"
          limits:
            cpu: "500m"
            memory: "512Mi"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
```

### Service Configuration

```yaml
# Complete service configuration
apiVersion: v1
kind: Service
metadata:
  name: drmp-gateway-service
spec:
  type: LoadBalancer
  ports:
  - port: 80
    targetPort: 8080
    name: http
  - port: 443
    targetPort: 8080
    name: https
  selector:
    app: gateway
```

## Docker Compose Deployment

### Prerequisites

- Docker 20.10+
- Docker Compose 2.0+

### Installation

1. **Configure docker-compose.yml**
   ```yaml
   version: '3.8'
   
   services:
     gateway:
       image: drmp/gateway:latest
       ports:
         - "8888:8080"
       environment:
         - DATABASE_URL=postgresql://postgres:password@postgres:5432/drmp
         - REDIS_URL=redis://redis:6379
         - GATEWAY_PORT=8080
       depends_on:
         - postgres
         - redis
       restart: unless-stopped
   
     media-edge:
       image: drmp/media-edge:latest
       ports:
         - "1935:1935"
         - "8081:8081"
       environment:
         - WEBRTC_PORT=8081
         - RTMP_PORT=1935
       depends_on:
         - gateway
       restart: unless-stopped
   
     sfu:
       image: drmp/sfu:latest
       ports:
         - "5004:5004"
       environment:
         - SFU_PORT=5004
       depends_on:
         - gateway
       restart: unless-stopped
   
     postgres:
       image: postgres:15
       environment:
         - POSTGRES_DB=drmp
         - POSTGRES_USER=postgres
         - POSTGRES_PASSWORD=password
       volumes:
         - postgres_data:/var/lib/postgresql/data
       restart: unless-stopped
   
     redis:
       image: redis:7-alpine
       restart: unless-stopped
   
     prometheus:
       image: prom/prometheus:latest
       ports:
         - "9090:9090"
       volumes:
         - ./prometheus.yml:/etc/prometheus/prometheus.yml
       restart: unless-stopped
   
     grafana:
       image: grafana/grafana:latest
       ports:
         - "3000:3000"
       environment:
         - GF_SECURITY_ADMIN_PASSWORD=admin
       restart: unless-stopped
   
   volumes:
     postgres_data:
   ```

2. **Start services**
   ```bash
   docker-compose up -d
   ```

### Scaling

```bash
# Scale services
docker-compose up -d --scale sfu=3 --scale media-edge=2
```

## Production Checklist

### Before Deployment

- [ ] Review security configuration
- [ ] Configure monitoring and alerting
- [ ] Set up backup and disaster recovery
- [ ] Test performance under load
- [ ] Verify compliance requirements
- [ ] Document deployment procedures

### Security

- [ ] Enable HTTPS everywhere
- [ ] Configure firewall rules
- [ ] Set up authentication and authorization
- [ ] Implement rate limiting
- [ ] Enable audit logging
- [ ] Regular security updates

### Performance

- [ ] Optimize database queries
- [ ] Configure caching strategies
- [ ] Tune media settings
- [ ] Set up CDN
- [ ] Monitor resource usage
- [ ] Implement auto-scaling

### Reliability

- [ ] Set up health checks
- [ ] Configure load balancing
- [ ] Implement circuit breakers
- [ ] Set up redundancy
- [ ] Test failover procedures
- [ ] Monitor uptime

### Monitoring

- [ ] Deploy Prometheus
- [ ] Set up Grafana dashboards
- [ ] Configure alerting rules
- [ ] Monitor logs
- [ ] Track performance metrics
- [ ] Set up error tracking

### Backup & Recovery

- [ ] Configure database backups
- [ ] Set up storage snapshots
- [ ] Test restore procedures
- [ ] Document recovery steps
- [ ] Monitor backup success
- [ ] Set retention policies

### Documentation

- [ ] Update deployment guides
- [ ] Document API changes
- [ ] Create troubleshooting guides
- [ ] Set up knowledge base
- [ ] Document maintenance procedures
- [ ] Create runbooks

---

**DRMP Deployment** - Comprehensive deployment options for every use case.