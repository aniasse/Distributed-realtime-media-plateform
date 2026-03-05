# DRMP - Documentation d'installation et démarrage

## Guide d'installation

### Prérequis

#### Système
- Linux (Ubuntu 20.04+ recommandé), macOS ou Windows 10+
- 8GB RAM minimum (16GB recommandé)
- 20GB d'espace disque libre
- CPU multi-cœur (4 cœurs minimum)

#### Logiciels requis
- **Docker & Docker Compose** (version 20.10+)
- **Rust** (version 1.70+)
- **PostgreSQL** (version 13+)
- **Kubernetes** (pour déploiement production)
- **Node.js** (version 18+ pour les exemples)

### Installation de Docker

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install docker.io docker-compose -y
sudo systemctl start docker
sudo systemctl enable docker
```

#### macOS
```bash
# Installer Docker Desktop depuis https://www.docker.com/products/docker-desktop
# Ou via Homebrew
brew install docker docker-compose
```

#### Windows
```bash
# Installer Docker Desktop depuis https://www.docker.com/products/docker-desktop
# Activer WSL 2 si nécessaire
```

### Installation de Rust

```bash
# Installer rustup
sudo apt install curl -y
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Ajouter rust à PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Installer la dernière version stable de Rust
rustup install stable
rustup default stable

# Vérifier l'installation
rustc --version
cargo --version
```

### Installation de PostgreSQL

#### Ubuntu/Debian
```bash
sudo apt install postgresql postgresql-contrib -y
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Créer un utilisateur et une base de données
sudo -u postgres psql -c "CREATE USER drmp WITH PASSWORD 'drmp123';"
sudo -u postgres psql -c "CREATE DATABASE drmp OWNER drmp;"
```

#### macOS
```bash
# Via Homebrew
brew install postgresql
brew services start postgresql

# Créer un utilisateur et une base de données
createdb drmp
createuser drmp --pwprompt
```

## Configuration du projet

### Cloner le repository

```bash
git clone https://github.com/your-org/drmp.git
cd drmp
```

### Configuration des variables d'environnement

Créer un fichier `.env` dans le répertoire racine :

```bash
# Copier le fichier d'exemple
cp .env.example .env

# Éditer le fichier .env
nano .env
```

Contenu du fichier `.env` :

```bash
# Base de données
DATABASE_URL=postgresql://drmp:drmp123@localhost:5432/drmp

# Redis
REDIS_URL=redis://localhost:6379

# JWT
JWT_SECRET=your-super-secret-jwt-key-change-in-production
TOKEN_EXPIRY_HOURS=24

# Media
MAX_PARTICIPANTS=100
MAX_BANDWIDTH_MBPS=10

# Recording
RECORDING_STORAGE_PATH=/recordings
RECORDING_RETENTION_DAYS=30

# Logging
RUST_LOG=info
LOG_LEVEL=info

# Monitoring
METRICS_PORT=9090
PROMETHEUS_SCRAPE_INTERVAL=30s

# Gateway
GATEWAY_PORT=8888
GATEWAY_HOST=0.0.0.0
```

## Démarrage en développement

### Option 1: Docker Compose (recommandé)

```bash
# Démarrer tous les services
docker-compose -f docker/docker-compose.dev.yml up --build

# Démarrer en arrière-plan
docker-compose -f docker/docker-compose.dev.yml up -d

# Vérifier les logs
docker-compose -f docker/docker-compose.dev.yml logs -f

# Arrêter les services
docker-compose -f docker/docker-compose.dev.yml down
```

### Option 2: Développement local

```bash
# Compiler et démarrer chaque service individuellement
cd services/gateway
cargo run

cd services/control-plane
cargo run

cd services/auth
cargo run

cd services/recording
cargo run

cd services/media-edge
cargo run

cd services/sfu
cargo run
```

### Option 3: Makefile (si disponible)

```bash
# Compiler tous les services
make build

# Démarrer tous les services
make dev

# Nettoyer les binaires
make clean
```

## Vérification de l'installation

### Tester les services

```bash
# Vérifier le gateway
curl http://localhost:8888/health

# Vérifier le control-plane
curl http://localhost:8080/health

# Vérifier l'auth service
curl http://localhost:8081/health

# Vérifier le recording service
curl http://localhost:8082/health

# Vérifier le media edge
curl http://localhost:1935
curl http://localhost:8081

# Vérifier le SFU
curl http://localhost:5004/health
```

### Tester la base de données

```bash
# Se connecter à PostgreSQL
sudo -u postgres psql -d drmp

# Vérifier les tables
\dt

# Quitter
\q
```

### Tester Redis

```bash
# Se connecter à Redis
redis-cli

# Tester la connexion
PING

# Quitter
EXIT
```

## Configuration avancée

### Configuration Kubernetes

```bash
# Appliquer la configuration Kubernetes de base
kubectl apply -f k8s/

# Vérifier les déploiements
kubectl get deployments

# Vérifier les services
kubectl get services

# Vérifier les pods
kubectl get pods
```

### Configuration de monitoring

```bash
# Démarrer Prometheus et Grafana
kubectl apply -f k8s/monitoring/

# Accéder à Grafana
# URL: http://localhost:3000
# Username: admin
# Password: admin

# Accéder à Prometheus
# URL: http://localhost:9090
```

### Configuration SSL/TLS

```bash
# Générer des certificats auto-signés
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes

# Configurer les services pour utiliser HTTPS
# Modifier les configurations des services pour utiliser les certificats
```

## Dépannage

### Problèmes courants

#### Port déjà utilisé
```bash
# Vérifier les processus utilisant les ports
sudo netstat -tulpn | grep :8888
sudo netstat -tulpn | grep :1935

# Tuer les processus
sudo kill -9 <PID>
```

#### Erreur de connexion à la base de données
```bash
# Vérifier PostgreSQL
sudo systemctl status postgresql

# Vérifier les credentials
cat .env | grep DATABASE_URL

# Tester la connexion
psql $DATABASE_URL -c "SELECT 1;"
```

#### Problème de permissions Docker
```bash
# Ajouter l'utilisateur au groupe docker
sudo usermod -aG docker $USER
# Se déconnecter et reconnecter
```

#### Compilation lente
```bash
# Utiliser le cache de compilation
cargo build --release

# Nettoyer le cache si nécessaire
cargo clean
```

### Logs et monitoring

```bash
# Logs Docker Compose
docker-compose -f docker/docker-compose.dev.yml logs -f

# Logs Kubernetes
kubectl logs -f deployment/gateway
kubectl logs -f deployment/control-plane

# Logs des services individuels
cd services/gateway
cargo watch -x run
```

## Validation finale

### Test de bout en bout

```bash
# Test de l'API Gateway
curl -X POST http://localhost:8888/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "test", "password": "test"}'

# Test de création de room
curl -X POST http://localhost:8888/api/control-plane/rooms \
  -H "Content-Type: application/json" \
  -d '{"tenant_id": "00000000-0000-0000-0000-000000000001", "max_participants": 10}'

# Test de streaming RTMP
ffmpeg -re -i test.mp4 -c copy -f flv rtmp://localhost:1935/live/test

# Test WebRTC (via navigateur)
# Ouvrir http://localhost:8888/webrtc.html
```

### Benchmarks

```bash
# Test de performance simple
ab -n 1000 -c 10 http://localhost:8888/health

# Test de charge
hey -n 1000 -c 10 http://localhost:8888/health

# Monitor les métriques
curl http://localhost:9090/api/v1/query?query=up
```

## Prochaines étapes

1. **Configuration de production** : Suivre le guide de déploiement production
2. **Sécurité** : Configurer SSL/TLS et RBAC
3. **Monitoring** : Configurer des dashboards Grafana
4. **Scaling** : Configurer l'auto-scaling Kubernetes
5. **Intégration** : Intégrer avec vos applications existantes