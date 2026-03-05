use super::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn, error, debug};
use tokio::time::{timeout, Duration};

pub struct AdaptiveBitrateController {
    config: ABRConfig,
    metrics: Arc<RwLock<ABRMetrics>>,
    logger: slog::Logger,
}

#[derive(Debug, Clone)]
pub struct ABRConfig {
    pub initial_bitrate_kbps: u32,
    pub max_bitrate_kbps: u32,
    pub min_bitrate_kbps: u32,
    pub adaptation_interval_ms: u64,
    pub buffer_size_ms: u32,
    pub reaction_window_ms: u32,
    pub downstep_p: f64,
    pub upstep_p: f64,
    pub probing_p: f64,
}

#[derive(Debug, Clone)]
pub struct ABRMetrics {
    pub current_bitrate_kbps: u32,
    pub target_bitrate_kbps: u32,
    pub buffer_level_ms: u32,
    pub packet_loss_percent: f64,
    pub latency_ms: f64,
    pub jitter_ms: f64,
    pub throughput_kbps: f64,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct SimulcastConfig {
    pub enabled: bool,
    pub layers: Vec<SimulcastLayerConfig>,
    pub adaptation_mode: SimulcastAdaptationMode,
}

#[derive(Debug, Clone)]
pub struct SimulcastLayerConfig {
    pub id: Uuid,
    pub bitrate_kbps: u32,
    pub resolution: (u32, u32),
    pub framerate: u32,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SimulcastAdaptationMode {
    QualityBased,
    BufferBased,
    Hybrid,
}

impl AdaptiveBitrateController {
    pub fn new(config: ABRConfig, logger: slog::Logger) -> Self {
        Self {
            config,
            metrics: Arc::new(RwLock::new(ABRMetrics {
                current_bitrate_kbps: config.initial_bitrate_kbps,
                target_bitrate_kbps: config.initial_bitrate_kbps,
                buffer_level_ms: config.buffer_size_ms,
                packet_loss_percent: 0.0,
                latency_ms: 0.0,
                jitter_ms: 0.0,
                throughput_kbps: config.initial_bitrate_kbps as f64,
                last_update: chrono::Utc::now(),
            })),
            logger,
        }
    }

    pub async fn initialize(&self) {
        info!(self.logger, "Initializing ABR controller";
              "config" => ?self.config);
    }

    pub async fn update_metrics(&self, packet_loss: f64, latency: f64, jitter: f64, throughput: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.packet_loss_percent = packet_loss;
        metrics.latency_ms = latency;
        metrics.jitter_ms = jitter;
        metrics.throughput_kbps = throughput;
        metrics.last_update = chrono::Utc::now();
        
        self.adapt_bitrate().await;
    }

    async fn adapt_bitrate(&self) {
        let mut metrics = self.metrics.write().await;
        
        // Calculate target bitrate based on network conditions
        let target_bitrate = self.calculate_target_bitrate(&metrics);
        
        // Apply bitrate limits
        let target_bitrate = target_bitrate.max(self.config.min_bitrate_kbps)
                                           .min(self.config.max_bitrate_kbps);
        
        // Smooth bitrate changes
        let new_bitrate = self.smooth_bitrate_change(metrics.current_bitrate_kbps, target_bitrate);
        
        metrics.current_bitrate_kbps = new_bitrate;
        metrics.target_bitrate_kbps = target_bitrate;
        
        debug!(self.logger, "ABR adaptation";
              "current_bitrate" => metrics.current_bitrate_kbps,
              "target_bitrate" => metrics.target_bitrate_kbps,
              "packet_loss" => metrics.packet_loss_percent,
              "latency" => metrics.latency_ms);
    }

    fn calculate_target_bitrate(&self, metrics: &ABRMetrics) -> u32 {
        // Simple throughput-based adaptation
        let throughput_factor = metrics.throughput_kbps / self.config.initial_bitrate_kbps as f64;
        
        // Packet loss penalty
        let loss_penalty = if metrics.packet_loss_percent > 5.0 {
            0.5
        } else if metrics.packet_loss_percent > 2.0 {
            0.7
        } else {
            1.0
        };
        
        // Latency penalty
        let latency_penalty = if metrics.latency_ms > 500.0 {
            0.6
        } else if metrics.latency_ms > 200.0 {
            0.8
        } else {
            1.0
        };
        
        let base_bitrate = self.config.initial_bitrate_kbps as f64;
        let adapted_bitrate = base_bitrate * throughput_factor * loss_penalty * latency_penalty;
        
        adapted_bitrate as u32
    }

    fn smooth_bitrate_change(&self, current: u32, target: u32) -> u32 {
        // Simple smoothing to avoid rapid changes
        if target > current {
            // Gradual increase
            current + ((target - current) / 2).min(500)
        } else {
            // Gradual decrease
            current - ((current - target) / 2).min(500)
        }
    }

    pub async fn get_current_bitrate(&self) -> u32 {
        self.metrics.read().await.current_bitrate_kbps
    }

    pub async fn get_target_bitrate(&self) -> u32 {
        self.metrics.read().await.target_bitrate_kbps
    }

    pub async fn get_buffer_level(&self) -> u32 {
        self.metrics.read().await.buffer_level_ms
    }

    pub async fn get_packet_loss(&self) -> f64 {
        self.metrics.read().await.packet_loss_percent
    }

    pub async fn get_latency(&self) -> f64 {
        self.metrics.read().await.latency_ms
    }

    pub async fn get_jitter(&self) -> f64 {
        self.metrics.read().await.jitter_ms
    }

    pub async fn get_throughput(&self) -> f64 {
        self.metrics.read().await.throughput_kbps
    }
}

// Simulcast controller
pub struct SimulcastController {
    config: SimulcastConfig,
    active_layers: Arc<RwLock<Vec<Uuid>>>,
    abr_controller: Arc<AdaptiveBitrateController>,
    logger: slog::Logger,
}

impl SimulcastController {
    pub fn new(config: SimulcastConfig, abr_controller: Arc<AdaptiveBitrateController>, logger: slog::Logger) -> Self {
        Self {
            config,
            active_layers: Arc::new(RwLock::new(Vec::new())),
            abr_controller,
            logger,
        }
    }

    pub async fn initialize(&self) {
        info!(self.logger, "Initializing simulcast controller";
              "config" => ?self.config);
        
        // Activate initial layers
        let mut active_layers = self.active_layers.write().await;
        active_layers.clear();
        
        for layer in self.config.layers.iter() {
            if layer.active {
                active_layers.push(layer.id);
            }
        }
    }

    pub async fn adapt_layers(&self) {
        let current_bitrate = self.abr_controller.get_current_bitrate().await;
        let target_bitrate = self.abr_controller.get_target_bitrate().await;
        
        // Calculate available bitrate for simulcast
        let available_bitrate = target_bitrate;
        
        // Select layers based on available bitrate
        let mut active_layers = Vec::new();
        let mut total_bitrate = 0;
        
        for layer in self.config.layers.iter() {
            if total_bitrate + layer.bitrate_kbps <= available_bitrate {
                active_layers.push(layer.id);
                total_bitrate += layer.bitrate_kbps;
            }
        }
        
        // Update active layers
        *self.active_layers.write().await = active_layers;
        
        debug!(self.logger, "Simulcast layer adaptation";
              "available_bitrate" => available_bitrate,
              "active_layers" => ?active_layers,
              "total_bitrate" => total_bitrate);
    }

    pub async fn get_active_layers(&self) -> Vec<Uuid> {
        self.active_layers.read().await.clone()
    }

    pub async fn get_layer_info(&self, layer_id: Uuid) -> Option<SimulcastLayerConfig> {
        self.config.layers.iter().find(|l| l.id == layer_id).cloned()
    }

    pub async fn set_layer_active(&self, layer_id: Uuid, active: bool) -> Result<(), String> {
        let mut layers = self.config.layers.clone();
        
        if let Some(layer) = layers.iter_mut().find(|l| l.id == layer_id) {
            layer.active = active;
            
            // Update configuration
            self.config.layers = layers;
            
            // Re-initialize
            self.initialize().await;
            
            Ok(())
        } else {
            Err("Layer not found".to_string())
        }
    }

    pub async fn add_layer(&self, layer: SimulcastLayerConfig) -> Result<(), String> {
        let mut layers = self.config.layers.clone();
        layers.push(layer);
        
        self.config.layers = layers;
        
        // Re-initialize
        self.initialize().await;
        
        Ok(())
    }

    pub async fn remove_layer(&self, layer_id: Uuid) -> Result<(), String> {
        let mut layers = self.config.layers.clone();
        layers.retain(|l| l.id != layer_id);
        
        self.config.layers = layers;
        
        // Re-initialize
        self.initialize().await;
        
        Ok(())
    }
}