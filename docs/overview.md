# DRMP - Plateforme de communication média temps réel distribuée

## Vue d'ensemble de l'architecture

DRMP (Distributed Realtime Media Platform) est une plateforme de communication média en temps réel, conçue pour être évolutive, distribuée et hautement performante. Construite avec Rust, elle offre une base de code sûre, concurrente et optimisée pour le traitement média.

### Architecture générale

```
┌───────────────────────────────────────────────────────────────────────────────────┐
│                    Gateway Service (8888)                    │
│  ┌───────────────────────────────────────────────────────────────────────────────────┘  │
│  │   Auth API    │ │ Control Plane │ │   Recording   │  │
│  │     (8081)     │ │     (8080)     │ │     (8080)     │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────────────────────┘
         │
         ▼
┌───────────────────────────────────────────────────────────────────────────────────┐
│                Media Edge Service (1935, 8081)               │
│  ┌───────────────────────────────────────────────────────────────────────────────────┘  │
│  │    RTMP       │ │   WebRTC     │  │
│  │    (1935)      │ │   (8081)      │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────────────────────┘
         │
         ▼
┌───────────────────────────────────────────────────────────────────────────────────┐
│                       SFU Cluster (5004)                     │
│  ┌───────────────────────────────────────────────────────────────────────────────────┘  │
│  │   Packet     │ │  Room Mgmt   │ │  Forwarding  │  │
│  │   Router     │ │   (API)      │ │   Engine     │  │
│  └───────────────────────────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────────────────────────┘
```

## Services principaux

### Gateway (port 8888)
Point d'entrée unique pour toutes les requêtes API. Route les requêtes vers les services appropriés et gère l'authentification.

### Control Plane (port 8080)
Service de gestion des pièces, des pairs et des pistes. Gère la logique métier et l'état de la session.

### Auth Service (port 8081)
Service d'authentification JWT avec support multi-fournisseur (OAuth2, LDAP, custom).

### Recording Service (port 8080)
Service d'enregistrement média avec stockage segmenté et plusieurs backends.

### Media Edge (ports 1935, 8081)
Gestion des protocoles RTMP et WebRTC, conversion de protocole et forwarding.

### SFU (Selective Forwarding Unit)
Moteur de forwarding de paquets avec support simulcast, SVC, et adaptive bitrate.

## Fonctionnalités clés

### Support multi-protocoles
- **WebRTC**: Vidéoconférence temps réel avec DTLS/SRTP
- **RTMP**: Streaming vers les plateformes existantes
- **HLS/DASH**: Streaming HTTP adaptatif
- **WebSocket**: Communication bidirectionnelle
- **SRT**: Streaming fiable sur réseaux non fiables

### Architecture distribuée
- **Microservices**: Services indépendants et évolutifs
- **Kubernetes native**: Déploiement production-ready
- **Multi-region**: Support edge computing
- **Auto-scaling**: Scaling horizontal et vertical

### Sécurité et authentification
- **JWT**: Authentication basée sur les jetons
- **RBAC**: Contrôle d'accès basé sur les rôles
- **SSL/TLS**: Chiffrement de bout en bout
- **Audit logging**: Suivi complet des actions

### Performance et optimisation
- **Adaptive Bitrate**: Qualité adaptative
- **Packet Loss Recovery**: NACK-based retransmission
- **Congestion Control**: Algorithmes TCP-friendly
- **Hardware Acceleration**: Offloading GPU/CPU

## Cas d'usage

### Vidéoconférence
- Réunions d'équipe
- Webinaires
- Éducation en ligne
- Télémédecine

### Streaming live
- Événements en direct
- Émissions de radio
- Gaming
- Sports

### Plateformes OTT
- Services de streaming
- Contenu à la demande
- TV en direct
- Plateformes d'apprentissage

## Technologies utilisées

### Backend
- **Rust**: Sécurité mémoire et performance
- **Tokio**: Runtime asynchrone
- **Actix Web**: Framework web
- **PostgreSQL**: Base de données relationnelle
- **Redis**: Cache et sessions

### Monitoring
- **Prometheus**: Métriques et monitoring
- **Grafana**: Visualisation des données
- **OpenTelemetry**: Tracing distribué

### Déploiement
- **Docker**: Conteneurisation
- **Kubernetes**: Orchestration
- **Helm**: Gestion de packages

## Licence

Ce projet est sous licence MIT. Voir le fichier LICENSE pour plus de détails.

## Contribution

Nous accueillons les contributions de la communauté. Voir le guide de contribution pour plus d'informations.