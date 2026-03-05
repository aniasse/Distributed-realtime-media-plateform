# DRMP - Guide d'installation et démarrage

## Configuration de base

### Structure du projet

```
drmp/
├── services/                 # Microservices
│   ├── gateway/             # Point d'entrée API
│   ├── control-plane/       # Gestion des pièces
│   ├── auth/                # Authentification
│   ├── recording/           # Enregistrement
│   ├── media-edge/          # Gestion des protocoles
│   └── sfu/                 # Moteur de forwarding
├── shared/                  # Bibliothèques partagées
├── docs/                    # Documentation
├── examples/                # Exemples pratiques
├── tutorials/               # Tutoriels
├── docker/                  # Configurations Docker
├── k8s/                     # Manifests Kubernetes
└── tests/                   # Tests automatisés
```

### Variables d'environnement

```bash
# Copier le fichier d'exemple
cp .env.example .env

# Éditer le fichier .env
nano .env
```

## Démarrage rapide

### Avec Docker Compose

```bash
# Démarrer tous les services
docker-compose -f docker/docker-compose.dev.yml up --build

# Accéder aux services
# Gateway API: http://localhost:8888
# Grafana: http://localhost:3000 (admin/admin)
# Prometheus: http://localhost:9090
```

### Développement local

```bash
# Compiler et démarrer chaque service
cd services/gateway
cargo run

cd services/control-plane
cargo run

# ... et ainsi de suite pour chaque service
```

## Vérification

```bash
# Tester les services
curl http://localhost:8888/health
curl http://localhost:8080/health
```

## Configuration avancée

### Kubernetes

```bash
# Déployer sur Kubernetes
kubectl apply -f k8s/

# Vérifier les déploiements
kubectl get deployments
```

### Monitoring

```bash
# Configurer Prometheus et Grafana
kubectl apply -f k8s/monitoring/
```

## Dépannage

### Problèmes courants

- **Ports déjà utilisés** : Vérifier avec `netstat`
- **Base de données** : Vérifier PostgreSQL et les credentials
- **Permissions** : Ajouter l'utilisateur au groupe docker

## Prochaines étapes

1. **Configuration de production**
2. **Sécurité**
3. **Monitoring**
4. **Scaling**
5. **Intégration**