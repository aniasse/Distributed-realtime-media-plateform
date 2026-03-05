use super::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn, error, debug};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtocolConfig {
    pub protocol: ProtocolType,
    pub enabled: bool,
    pub settings: HashMap<String, String>,
    pub quality_preset: QualityPreset,
    pub network_profile: NetworkProfile,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QualityPreset {
    pub name: String,
    pub video_bitrate_kbps: u32,
    pub audio_bitrate_kbps: u32,
    pub resolution: (u32, u32),
    pub framerate: u32,
    pub keyframe_interval: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkProfile {
    pub name: String,
    pub max_latency_ms: u32,
    pub max_packet_loss: f64,
    pub max_jitter_ms: u32,
    pub buffer_size_ms: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtocolManagerConfig {
    pub protocols: HashMap<ProtocolType, ProtocolConfig>,
    pub quality_presets: HashMap<String, QualityPreset>,
    pub network_profiles: HashMap<String, NetworkProfile>,
    pub default_protocol: ProtocolType,
    pub default_quality_preset: String,
    pub default_network_profile: String,
}

pub struct ConfigurationSystem {
    config: Arc<RwLock<ProtocolManagerConfig>>,
    logger: slog::Logger,
}

impl ConfigurationSystem {
    pub fn new(logger: slog::Logger) -> Self {
        Self {
            config: Arc::new(RwLock::new(Self::default_config())),
            logger,
        }
    }

    fn default_config() -> ProtocolManagerConfig {
        ProtocolManagerConfig {
            protocols: HashMap::from([
                (ProtocolType::WebRTC, ProtocolConfig {
                    protocol: ProtocolType::WebRTC,
                    enabled: true,
                    settings: HashMap::from([
                        ("stun_servers".to_string(), "stun:stun.l.google.com:19302".to_string()),
                        ("turn_servers".to_string(), "turn:turn.example.com:3478".to_string()),
                        ("dtls".to_string(), "true".to_string()),
                    ]),
                    quality_preset: QualityPreset::Medium,
                    network_profile: NetworkProfile::Average,
                }),
                (ProtocolType::RTMP, ProtocolConfig {
                    protocol: ProtocolType::RTMP,
                    enabled: true,
                    settings: HashMap::from([
                        ("port".to_string(), "1935".to_string()),
                        ("chunk_size".to_string(), "4096".to_string()),
                    ]),
                    quality_preset: QualityPreset::Medium,
                    network_profile: NetworkProfile::Average,
                }),
                (ProtocolType::HLS, ProtocolConfig {
                    protocol: ProtocolType::HLS,
                    enabled: true,
                    settings: HashMap::from([
                        ("segment_duration".to_string(), "6".to_string()),
                        ("output_dir".to_string(), "./hls".to_string()),
                    ]),
                    quality_preset: QualityPreset::Medium,
                    network_profile: NetworkProfile::Good,
                }),
                (ProtocolType::DASH, ProtocolConfig {
                    protocol: ProtocolType::DASH,
                    enabled: true,
                    settings: HashMap::from([
                        ("segment_duration".to_string(), "2".to_string()),
                        ("output_dir".to_string(), "./dash".to_string()),
                    ]),
                    quality_preset: QualityPreset::Medium,
                    network_profile: NetworkProfile::Good,
                }),
                (ProtocolType::SRT, ProtocolConfig {
                    protocol: ProtocolType::SRT,
                    enabled: true,
                    settings: HashMap::from([
                        ("port".to_string(), "5000".to_string()),
                        ("latency".to_string(), "120".to_string()),
                        ("packet_loss".to_string(), "0.1".to_string()),
                    ]),
                    quality_preset: QualityPreset::Medium,
                    network_profile: NetworkProfile::Poor,
                }),
                (ProtocolType::WebSocket, ProtocolConfig {
                    protocol: ProtocolType::WebSocket,
                    enabled: true,
                    settings: HashMap::from([
                        ("port".to_string(), "8080".to_string()),
                        ("path".to_string(), "/ws".to_string()),
                    ]),
                    quality_preset: QualityPreset::Medium,
                    network_profile: NetworkProfile::Average,
                }),
            ]),
            quality_presets: HashMap::from([
                ("low".to_string(), QualityPreset {
                    name: "low".to_string(),
                    video_bitrate_kbps: 500,
                    audio_bitrate_kbps: 64,
                    resolution: (640, 360),
                    framerate: 15,
                    keyframe_interval: 60,
                }),
                ("medium".to_string(), QualityPreset {
                    name: "medium".to_string(),
                    video_bitrate_kbps: 2000,
                    audio_bitrate_kbps: 128,
                    resolution: (1280, 720),
                    framerate: 30,
                    keyframe_interval: 60,
                }),
                ("high".to_string(), QualityPreset {
                    name: "high".to_string(),
                    video_bitrate_kbps: 4000,
                    audio_bitrate_kbps: 192,
                    resolution: (1920, 1080),
                    framerate: 30,
                    keyframe_interval: 60,
                }),
                ("ultra".to_string(), QualityPreset {
                    name: "ultra".to_string(),
                    video_bitrate_kbps: 8000,
                    audio_bitrate_kbps: 256,
                    resolution: (3840, 2160),
                    framerate: 60,
                    keyframe_interval: 60,
                }),
            ]),
            network_profiles: HashMap::from([
                ("good".to_string(), NetworkProfile {
                    name: "good".to_string(),
                    max_latency_ms: 100,
                    max_packet_loss: 0.01,
                    max_jitter_ms: 20,
                    buffer_size_ms: 1000,
                }),
                ("average".to_string(), NetworkProfile {
                    name: "average".to_string(),
                    max_latency_ms: 200,
                    max_packet_loss: 0.02,
                    max_jitter_ms: 50,
                    buffer_size_ms: 2000,
                }),
                ("poor".to_string(), NetworkProfile {
                    name: "poor".to_string(),
                    max_latency_ms: 500,
                    max_packet_loss: 0.05,
                    max_jitter_ms: 100,
                    buffer_size_ms: 4000,
                }),
                ("very_poor".to_string(), NetworkProfile {
                    name: "very_poor".to_string(),
                    max_latency_ms: 1000,
                    max_packet_loss: 0.1,
                    max_jitter_ms: 200,
                    buffer_size_ms: 8000,
                }),
            ]),
            default_protocol: ProtocolType::WebRTC,
            default_quality_preset: "medium".to_string(),
            default_network_profile: "average".to_string(),
        }
    }

    pub async fn get_protocol_config(&self, protocol_type: &ProtocolType) -> Option<ProtocolConfig> {
        let config = self.config.read().await;
        config.protocols.get(protocol_type).cloned()
    }

    pub async fn set_protocol_config(&self, protocol_type: ProtocolType, config: ProtocolConfig) -> Result<(), String> {
        let mut config_lock = self.config.write().await;
        
        if config_lock.protocols.contains_key(&protocol_type) {
            config_lock.protocols.insert(protocol_type, config);
            Ok(())
        } else {
            Err("Protocol not found".to_string())
        }
    }

    pub async fn get_quality_preset(&self, name: &str) -> Option<QualityPreset> {
        let config = self.config.read().await;
        config.quality_presets.get(name).cloned()
    }

    pub async fn set_quality_preset(&self, name: &str, preset: QualityPreset) -> Result<(), String> {
        let mut config_lock = self.config.write().await;
        
        if config_lock.quality_presets.contains_key(name) {
            config_lock.quality_presets.insert(name.to_string(), preset);
            Ok(())
        } else {
            Err("Quality preset not found".to_string())
        }
    }

    pub async fn get_network_profile(&self, name: &str) -> Option<NetworkProfile> {
        let config = self.config.read().await;
        config.network_profiles.get(name).cloned()
    }

    pub async fn set_network_profile(&self, name: &str, profile: NetworkProfile) -> Result<(), String> {
        let mut config_lock = self.config.write().await;
        
        if config_lock.network_profiles.contains_key(name) {
            config_lock.network_profiles.insert(name.to_string(), profile);
            Ok(())
        } else {
            Err("Network profile not found".to_string())
        }
    }

    pub async fn get_all_protocols(&self) -> Vec<ProtocolConfig> {
        let config = self.config.read().await;
        config.protocols.values().cloned().collect()
    }

    pub async fn get_all_quality_presets(&self) -> Vec<QualityPreset> {
        let config = self.config.read().await;
        config.quality_presets.values().cloned().collect()
    }

    pub async fn get_all_network_profiles(&self) -> Vec<NetworkProfile> {
        let config = self.config.read().await;
        config.network_profiles.values().cloned().collect()
    }

    pub async fn get_default_protocol(&self) -> ProtocolType {
        let config = self.config.read().await;
        config.default_protocol.clone()
    }

    pub async fn set_default_protocol(&self, protocol_type: ProtocolType) -> Result<(), String> {
        let mut config_lock = self.config.write().await;
        
        if config_lock.protocols.contains_key(&protocol_type) {
            config_lock.default_protocol = protocol_type;
            Ok(())
        } else {
            Err("Protocol not found".to_string())
        }
    }

    pub async fn get_default_quality_preset(&self) -> String {
        let config = self.config.read().await;
        config.default_quality_preset.clone()
    }

    pub async fn set_default_quality_preset(&self, name: &str) -> Result<(), String> {
        let mut config_lock = self.config.write().await;
        
        if config_lock.quality_presets.contains_key(name) {
            config_lock.default_quality_preset = name.to_string();
            Ok(())
        } else {
            Err("Quality preset not found".to_string())
        }
    }

    pub async fn get_default_network_profile(&self) -> String {
        let config = self.config.read().await;
        config.default_network_profile.clone()
    }

    pub async fn set_default_network_profile(&self, name: &str) -> Result<(), String> {
        let mut config_lock = self.config.write().await;
        
        if config_lock.network_profiles.contains_key(name) {
            config_lock.default_network_profile = name.to_string();
            Ok(())
        } else {
            Err("Network profile not found".to_string())
        }
    }

    pub async fn validate_configuration(&self) -> Result<(), String> {
        let config = self.config.read().await;
        
        // Validate protocols
        for (protocol_type, protocol_config) in config.protocols.iter() {
            if !protocol_config.enabled && protocol_type == &config.default_protocol {
                return Err(format!("Default protocol {} is disabled", protocol_type));
            }
            
            // Validate quality preset
            if !config.quality_presets.contains_key(&protocol_config.quality_preset.name) {
                return Err(format!("Quality preset {} for protocol {} not found", 
                                   protocol_config.quality_preset.name, protocol_type));
            }
            
            // Validate network profile
            if !config.network_profiles.contains_key(&protocol_config.network_profile.name) {
                return Err(format!("Network profile {} for protocol {} not found", 
                                   protocol_config.network_profile.name, protocol_type));
            }
        }
        
        // Validate default quality preset
        if !config.quality_presets.contains_key(&config.default_quality_preset) {
            return Err(format!("Default quality preset {} not found", config.default_quality_preset));
        }
        
        // Validate default network profile
        if !config.network_profiles.contains_key(&config.default_network_profile) {
            return Err(format!("Default network profile {} not found", config.default_network_profile));
        }
        
        Ok(())
    }

    pub async fn save_to_file(&self, file_path: &str) -> Result<(), String> {
        let config = self.config.read().await;
        
        let config_str = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
        
        tokio::fs::write(file_path, config_str).await.map_err(|e| e.to_string())?;
        
        info!(self.logger, "Configuration saved to file";
              "file_path" => file_path);
        
        Ok(())
    }

    pub async fn load_from_file(&self, file_path: &str) -> Result<(), String> {
        let config_str = tokio::fs::read_to_string(file_path).await.map_err(|e| e.to_string())?;
        let config: ProtocolManagerConfig = serde_json::from_str(&config_str).map_err(|e| e.to_string())?;
        
        *self.config.write().await = config;
        
        info!(self.logger, "Configuration loaded from file";
              "file_path" => file_path);
        
        Ok(())
    }
}