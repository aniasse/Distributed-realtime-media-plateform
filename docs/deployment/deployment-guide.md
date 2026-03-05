# DRMP - Guide de déploiement

## Vue d'ensemble

DRMP peut être déployé de plusieurs manières selon les besoins :

- **Développement local** : Docker Compose
- **Production** : Kubernetes
- **Cloud** : AWS, GCP, Azure
- **On-premise** : Serveurs physiques/virtuels
- **Edge computing** : Déploiement régional

## Déploiement local avec Docker Compose

### Configuration requise

```bash
# Vérifier Docker et Docker Compose
docker --version
docker-compose --version

# Vérifier que Docker est en cours d'exécution
sudo systemctl status docker
```

### Fichiers de configuration

```bash
# Fichier principal
docker-compose.yml

# Configuration développement
docker/docker-compose.dev.yml

# Configuration production
docker/docker-compose.prod.yml
```

### Variables d'environnement

```bash
# Copier le fichier d'exemple
cp .env.example .env

# Éditer le fichier .env
nano .env
```

### Démarrage des services

```bash
# Démarrage avec configuration développement
docker-compose -f docker/docker-compose.dev.yml up --build

# Démarrage en arrière-plan
docker-compose -f docker/docker-compose.dev.yml up -d

# Suivre les logs
docker-compose -f docker/docker-compose.dev.yml logs -f

# Arrêter les services
docker-compose -f docker/docker-compose.dev.yml down
```

### Vérification

```bash
# Vérifier les services en cours d'exécution
docker-compose -f docker/docker-compose.dev.yml ps

# Vérifier les logs
docker-compose -f docker/docker-compose.dev.yml logs gateway
docker-compose -f docker/docker-compose.dev.yml logs sfu

# Tester l'API
curl http://localhost:8888/health
```

## Déploiement Kubernetes

### Configuration requise

```bash
# Vérifier kubectl
kubectl version

# Vérifier le cluster
kubectl cluster-info
kubectl get nodes
```

### Structure des manifests

```
k8s/
├── namespace.yaml              # Espace de nom
├── configmap.yaml              # Configuration
├── secret.yaml                 # Secrets
├── ingress.yaml                # Ingress
├── hpa.yaml                    # Horizontal Pod Autoscaler
├── node-exporter.yaml          # Monitoring
├── database.yaml               # Base de données
├── gateway.yaml                # Service gateway
├── control-plane.yaml          # Service control plane
├── auth.yaml                   # Service auth
├── recording.yaml              # Service recording
├── media-edge.yaml             # Service media edge
├── sfu.yaml                    # Service SFU
└── monitoring/                 # Monitoring complet
    ├── prometheus.yaml
    ├── grafana.yaml
    └── dashboards/
```

### Déploiement de base

```bash
# Créer l'espace de nom
kubectl apply -f k8s/namespace.yaml

# Créer les secrets
kubectl apply -f k8s/secret.yaml

# Créer la configuration
kubectl apply -f k8s/configmap.yaml

# Déployer les services
kubectl apply -f k8s/

# Vérifier les déploiements
kubectl get deployments -n drmp
kubectl get pods -n drmp
kubectl get services -n drmp
```

### Configuration de la base de données

```bash
# Déployer PostgreSQL
kubectl apply -f k8s/database.yaml

# Vérifier le déploiement
kubectl get pods -n drmp -l app=postgresql

# Se connecter à la base de données
kubectl exec -it <postgresql-pod> -n drmp -- psql -U drmp -d drmp
```

### Configuration Ingress

```bash
# Déployer l'Ingress
kubectl apply -f k8s/ingress.yaml

# Vérifier l'Ingress
kubectl get ingress -n drmp

# Configurer le DNS (si nécessaire)
# drmp.example.com -> Ingress IP
```

### Scaling et haute disponibilité

```bash
# Configurer l'auto-scaling
kubectl apply -f k8s/hpa.yaml

# Vérifier l'auto-scaling
kubectl get hpa -n drmp

# Scaling manuel
kubectl scale deployment/gateway --replicas=3 -n drmp
kubectl scale deployment/sfu --replicas=5 -n drmp
```

## Déploiement cloud

### AWS (Amazon Web Services)

#### Prérequis

```bash
# Installer AWS CLI
curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
unzip awscliv2.zip
sudo ./aws/install

# Configurer AWS CLI
aws configure
```

#### Déploiement EKS

```bash
# Créer un cluster EKS
eksctl create cluster \
  --name drmp-cluster \
  --region us-west-2 \
  --nodegroup-name standard-workers \
  --node-type t3.medium \
  --nodes 3 \
  --nodes-min 1 \
  --nodes-max 6 \
  --managed

# Configurer kubectl pour le cluster
aws eks update-kubeconfig --region us-west-2 --name drmp-cluster

# Déployer DRMP
kubectl apply -f k8s/
```

#### Services AWS

```bash
# Base de données RDS
aws rds create-db-instance \
  --db-instance-identifier drmp-db \
  --db-instance-class db.t3.medium \
  --engine postgres \
  --master-username drmp \
  --master-user-password your_password \
  --allocated-storage 20

# Redis ElastiCache
aws elasticache create-cache-cluster \
  --cache-cluster-id drmp-redis \
  --engine redis \
  --cache-node-type cache.t3.micro \
  --num-cache-nodes 1
```

### Google Cloud Platform (GCP)

#### Prérequis

```bash
# Installer gcloud
curl https://sdk.cloud.google.com | bash
exec -l $SHELL

# Configurer gcloud
gcloud init
```

#### Déploiement GKE

```bash
# Créer un cluster GKE
gcloud container clusters create drmp-cluster \
  --zone us-central1-a \
  --machine-type n1-standard-2 \
  --num-nodes 3 \
  --enable-autoscaling \
  --min-nodes 1 \
  --max-nodes 6

# Configurer kubectl
gcloud container clusters get-credentials drmp-cluster --zone us-central1-a

# Déployer DRMP
kubectl apply -f k8s/
```

#### Services GCP

```bash
# Base de données Cloud SQL
gcloud sql instances create drmp-db \
  --database-version POSTGRES_13 \
  --tier db-custom-1-3840 \
  --region us-central1

# Memorystore Redis
gcloud redis instances create drmp-redis \
  --size=1 \
  --region=us-central1 \
  --redis-version=redis_6_x
```

### Microsoft Azure

#### Prérequis

```bash
# Installer Azure CLI
curl -sL https://aka.ms/InstallAzureCLIDeb | sudo bash

# Se connecter à Azure
az login
```

#### Déploiement AKS

```bash
# Créer un cluster AKS
az group create --name drmp-group --location eastus
az aks create --resource-group drmp-group --name drmp-cluster \
  --node-count 3 --enable-addons monitoring \
  --generate-ssh-keys

# Configurer kubectl
az aks get-credentials --resource-group drmp-group --name drmp-cluster

# Déployer DRMP
kubectl apply -f k8s/
```

#### Services Azure

```bash
# Base de données Azure Database for PostgreSQL
az postgres server create \
  --resource-group drmp-group \
  --name drmpdb \
  --location eastus \
  --admin-user drmp \
  --admin-password your_password \
  --sku-name B_Gen5_1

# Azure Cache for Redis
az redis create \
  --resource-group drmp-group \
  --name drmpredis \
  --location eastus \
  --sku Basic \
  --vm-size c0
```

## Déploiement multi-nœuds

### Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Load Balancer                         │
├─────────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                   Gateway Cluster                      │  │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │  │
│  │  │  Gateway Pod 1  │  Gateway Pod 2  │  Gateway Pod 3  │  │  │
│  │  └──────────────────────────────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                   Control Plane Cluster                │  │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │  │
│  │  │  Control Plane Pod 1  │  Control Plane Pod 2  │  Control Plane Pod 3  │  │  │
│  │  └──────────────────────────────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                   Media Edge Cluster                   │  │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │  │
│  │  │  Media Edge Pod 1  │  Media Edge Pod 2  │  Media Edge Pod 3  │  │  │
│  │  └──────────────────────────────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                   SFU Cluster                          │  │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │  │
│  │  │  SFU Pod 1  │  SFU Pod 2  │  SFU Pod 3  │  SFU Pod 4  │  │  │
│  │  └──────────────────────────────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

### Configuration des services

```yaml
# Gateway
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: gateway
        resources:
          requests:
            cpu: 500m
            memory: 512Mi
          limits:
            cpu: 1
            memory: 1Gi

# Control Plane
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: control-plane
        resources:
          requests:
            cpu: 500m
            memory: 512Mi
          limits:
            cpu: 1
            memory: 1Gi

# Media Edge
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: media-edge
        resources:
          requests:
            cpu: 1
            memory: 1Gi
          limits:
            cpu: 2
            memory: 2Gi

# SFU
spec:
  replicas: 4
  template:
    spec:
      containers:
      - name: sfu
        resources:
          requests:
            cpu: 2
            memory: 2Gi
          limits:
            cpu: 4
            memory: 4Gi
```

### Load Balancer

```yaml
# Service LoadBalancer
spec:
  type: LoadBalancer
  ports:
  - port: 80
    targetPort: 8888
    protocol: TCP
    name: http
  - port: 443
    targetPort: 8888
    protocol: TCP
    name: https
  selector:
    app: gateway
```

## Déploiement on-premise

### Architecture on-premise

```
┌─────────────────────────────────────────────────────────────────────┐
│                 On-Premise DRMP                          │
├─────────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                 Kubernetes Cluster                     │  │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │  │
│  │  │  Master Node 1  │  Master Node 2  │  Master Node 3  │  │  │  │
│  │  │  Worker Node 1  │  Worker Node 2  │  Worker Node 3  │  │  │  │
│  │  └──────────────────────────────────────────────────────────────────────┘  │  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │  │
│  ┌──────────────────────────────────────────────────────────────────────┐  │  │
│  │                 Storage Backend                        │  │  │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │  │  │
│  │  │  NAS/SAN Storage   │  Local SSD Storage  │  Backup Storage   │  │  │  │
│  │  └──────────────────────────────────────────────────────────────────────┘  │  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │  │
│  ┌──────────────────────────────────────────────────────────────────────┐  │  │
│  │                 Network Infrastructure                   │  │  │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │  │  │
│  │  │  Load Balancer    │  Firewall         │  VPN Gateway      │  │  │  │
│  │  └──────────────────────────────────────────────────────────────────────┘  │  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │  │
│  ┌──────────────────────────────────────────────────────────────────────┐  │  │
│  │                 Monitoring Stack                      │  │  │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │  │  │
│  │  │  Prometheus       │  Grafana         │  AlertManager     │  │  │  │
│  │  └──────────────────────────────────────────────────────────────────────┘  │  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │  │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

### Configuration on-premise

```yaml
# Base de données
database:
  host: postgres.internal
  port: 5432
  username: drmp
  password: your_password
  dbname: drmp

# Redis
redis:
  host: redis.internal
  port: 6379
  password: your_redis_password

# Stockage
storage:
  backend: local
  path: /mnt/storage/drmp
  backup_path: /mnt/backup/drmp

# Network
network:
  domain: internal.company.com
  load_balancer_ip: 192.168.1.100
  subnet: 192.168.1.0/24
```

### Sécurité on-premise

```yaml
# Firewall
firewall:
  rules:
    - port: 80
      protocol: tcp
      source: 0.0.0.0/0
    - port: 443
      protocol: tcp
      source: 0.0.0.0/0
    - port: 8888
      protocol: tcp
      source: 192.168.1.0/24
    - port: 1935
      protocol: tcp
      source: 192.168.1.0/24

# VPN
vpn:
  enabled: true
  server: vpn.company.com
  protocol: openvpn
  port: 1194

# Monitoring
monitoring:
  internal: true
  external_access: false
  retention_days: 90
```

## Déploiement edge computing

### Architecture edge

```
┌─────────────────────────────────────────────────────────────────────┐
│                 Edge Computing DRMP                     │
├─────────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                 Central Cloud                          │  │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │  │
│  │  │  Control Plane     │  Auth Service      │  Database         │  │  │  │
│  │  └──────────────────────────────────────────────────────────────────────┘  │  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │  │
│  ┌──────────────────────────────────────────────────────────────────────┐  │  │
│  │                 Edge Node 1 (US-East)                   │  │  │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │  │  │
│  │  │  Media Edge        │  SFU (partial)     │  Cache             │  │  │  │
│  │  └──────────────────────────────────────────────────────────────────────┘  │  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │  │
│  ┌──────────────────────────────────────────────────────────────────────┐  │  │
│  │                 Edge Node 2 (EU-West)                   │  │  │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │  │  │
│  │  │  Media Edge        │  SFU (partial)     │  Cache             │  │  │  │
│  │  └──────────────────────────────────────────────────────────────────────┘  │  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │  │
│  ┌──────────────────────────────────────────────────────────────────────┐  │  │
│  │                 Edge Node 3 (APAC)                     │  │  │
│  │  ┌──────────────────────────────────────────────────────────────────────┐  │  │  │
│  │  │  Media Edge        │  SFU (partial)     │  Cache             │  │  │  │
│  │  └──────────────────────────────────────────────────────────────────────┘  │  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │  │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘
```

### Configuration edge

```yaml
# Edge node configuration
edge:
  enabled: true
  region: us-east
  role: media-edge
  
  # Central connection
  central:
    api_url: https://drmp.company.com/api
    auth_token: your_edge_token
    
  # Local services
  local_services:
    media_edge:
      enabled: true
      port: 1935
      max_connections: 100
    
    sfu:
      enabled: true
      port: 5004
      max_rooms: 50
      
    cache:
      enabled: true
      max_size: 1000
      ttl: 300

# Synchronization
sync:
  enabled: true
  interval: 60
  conflict_resolution: last-write-wins

# Monitoring
edge_monitoring:
  enabled: true
  metrics_port: 9091
  send_to_central: true
  
# Security
edge_security:
  tls_enabled: true
  certificate_path: /etc/ssl/certs/edge.crt
  private_key_path: /etc/ssl/private/edge.key
```

## Validation du déploiement

### Tests de base

```bash
# Vérifier tous les services
kubectl get pods -n drmp --watch

# Vérifier les logs
docker-compose logs -f
docker logs <container_id>

# Tester l'API
curl http://localhost:8888/health
curl https://drmp.company.com/health

# Tester le streaming
ffmpeg -re -i test.mp4 -c copy -f flv rtmp://localhost:1935/live/test
```

### Tests de performance

```bash
# Test de charge
ab -n 1000 -c 10 http://localhost:8888/health
hey -n 1000 -c 10 http://localhost:8888/health

# Test de streaming
ffmpeg -re -i test.mp4 -c copy -f flv rtmp://localhost:1935/live/test
```

### Monitoring

```bash
# Vérifier les métriques
curl http://localhost:9090/api/v1/query?query=up

# Vérifier Grafana
open http://localhost:3000

# Vérifier les logs
kubectl logs -f deployment/gateway -n drmp
```

## Dépannage

### Problèmes courants

#### Service non démarré
```bash
# Vérifier l'état
docker-compose ps
kubectl get pods -n drmp

# Vérifier les logs
docker-compose logs <service>
kubectl logs <pod> -n drmp

# Redémarrer le service
docker-compose restart <service>
kubectl rollout restart deployment/<service> -n drmp
```

#### Erreur de connexion à la base de données
```bash
# Vérifier PostgreSQL
sudo systemctl status postgresql
kubectl get pods -n drmp -l app=postgresql

# Tester la connexion
psql $DATABASE_URL -c "SELECT 1;"
kubectl exec -it <postgresql-pod> -n drmp -- psql -U drmp -d drmp
```

#### Problème de réseau
```bash
# Vérifier les ports
sudo netstat -tulpn | grep :8888
sudo netstat -tulpn | grep :1935

# Vérifier le firewall
iptables -L -n
ufw status
```

#### Problème de permissions
```bash
# Permissions Docker
sudo usermod -aG docker $USER
# Se déconnecter et reconnecter

# Permissions Kubernetes
kubectl auth can-i get pods -n drmp
```

### Outils de diagnostic

```bash
# Vérifier les ressources
kubectl top pods -n drmp
kubectl describe pod <pod> -n drmp

# Vérifier les événements
kubectl get events -n drmp
kubectl describe deployment <deployment> -n drmp

# Debug container
kubectl exec -it <pod> -n drmp -- /bin/bash
```

## Scalabilité et optimisation

### Scaling horizontal

```bash
# Scaling manuel
kubectl scale deployment/gateway --replicas=5 -n drmp
kubectl scale deployment/sfu --replicas=10 -n drmp

# Scaling automatique
kubectl apply -f k8s/hpa.yaml
kubectl get hpa -n drmp
```

### Optimisations

```bash
# Resource limits
docker-compose -f docker/docker-compose.prod.yml up --build
kubectl apply -f k8s/resources.yaml

# Cache configuration
kubectl apply -f k8s/cache.yaml

# Monitoring optimization
kubectl apply -f k8s/monitoring-optimized.yaml
```

## Migration depuis une version précédente

### Sauvegarde

```bash
# Sauvegarder la base de données
pg_dump drmp > drmp_backup.sql

# Sauvegarder la configuration
kubectl get configmap drmp-config -n drmp -o yaml > drmp-config.yaml
kubectl get secret drmp-secrets -n drmp -o yaml > drmp-secrets.yaml
```

### Migration

```bash
# Arrêter les services
docker-compose down
kubectl delete -f k8s/

# Mettre à jour les images
docker pull drmp/drmp:latest

# Déployer la nouvelle version
docker-compose -f docker/docker-compose.prod.yml up --build
kubectl apply -f k8s/

# Restaurer la base de données
psql drmp < drmp_backup.sql
```

## Documentation complémentaire

- [Configuration détaillée](configuration.md)
- [Référence API](api.md)
- [Guide de sécurité](security.md)
- [Optimisation des performances](performance.md)