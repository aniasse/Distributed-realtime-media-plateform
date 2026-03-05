use log::{Level, LevelFilter};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Logger {
    level: LogLevel,
    component: String,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Logger {
    pub fn new(component: &str) -> Self {
        Self {
            level: LogLevel::Info,
            component: component.to_string(),
        }
    }

    pub fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    pub fn trace(&self, message: &str) {
        if self.level <= LogLevel::Trace {
            log::trace!("[{}] {}", self.component, message);
        }
    }

    pub fn debug(&self, message: &str) {
        if self.level <= LogLevel::Debug {
            log::debug!("[{}] {}", self.component, message);
        }
    }

    pub fn info(&self, message: &str) {
        if self.level <= LogLevel::Info {
            log::info!("[{}] {}", self.component, message);
        }
    }

    pub fn warn(&self, message: &str) {
        if self.level <= LogLevel::Warn {
            log::warn!("[{}] {}", self.component, message);
        }
    }

    pub fn error(&self, message: &str) {
        if self.level <= LogLevel::Error {
            log::error!("[{}] {}", self.component, message);
        }
    }
}

pub struct Metrics {
    counters: HashMap<String, u64>,
    gauges: HashMap<String, i64>,
    histograms: HashMap<String, Vec<f64>>,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
        }
    }

    pub fn increment_counter(&mut self, name: &str, value: u64) {
        *self.counters.entry(name.to_string()).or_insert(0) += value;
    }

    pub fn decrement_counter(&mut self, name: &str, value: u64) {
        *self.counters.entry(name.to_string()).or_insert(0) -= value;
    }

    pub fn set_gauge(&mut self, name: &str, value: i64) {
        self.gauges.insert(name.to_string(), value);
    }

    pub fn record_histogram(&mut self, name: &str, value: f64) {
        self.histograms
            .entry(name.to_string())
            .or_insert(Vec::new())
            .push(value);
    }

    pub fn get_counter(&self, name: &str) -> Option<u64> {
        self.counters.get(name).copied()
    }

    pub fn get_gauge(&self, name: &str) -> Option<i64> {
        self.gauges.get(name).copied()
    }

    pub fn get_histogram(&self, name: &str) -> Option<&Vec<f64>> {
        self.histograms.get(name)
    }
}

pub struct ErrorHandler {
    logger: Logger,
}

impl ErrorHandler {
    pub fn new(component: &str) -> Self {
        Self {
            logger: Logger::new(component),
        }
    }

    pub fn handle_error(&self, error: &dyn std::error::Error, context: &str) {
        self.logger
            .error(&format!("Error in {}: {}", context, error));
        self.logger.error(&format!("Backtrace: {:?}", error));
    }

    pub fn handle_panic(&self, info: &std::panic::PanicInfo) {
        let payload = info.payload();
        let message = if let Some(s) = payload.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        self.logger.error(&format!("Panic occurred: {}", message));
        if let Some(location) = info.location() {
            self.logger.error(&format!(
                "Location: {}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            ));
        }
    }
}

pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    let nanos = duration.subsec_nanos();

    if secs >= 3600 {
        format!("{}h {}m {}s", secs / 3600, (secs % 3600) / 60, secs % 60)
    } else if secs >= 60 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs > 0 || nanos >= 1_000_000 {
        format!(
            "{}s {}.{:03}ms",
            secs,
            nanos / 1_000_000,
            (nanos % 1_000_000) / 1_000
        )
    } else {
        format!("{}ns", nanos)
    }
}

pub fn hash_password(password: &str) -> String {
    use bcrypt::{hash, DEFAULT_COST};
    hash(password, DEFAULT_COST).unwrap_or_else(|_| "".to_string())
}

pub fn verify_password(password: &str, hashed: &str) -> bool {
    use bcrypt::verify;
    verify(password, hashed).unwrap_or(false)
}

pub fn generate_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn generate_stream_key() -> String {
    format!("live_{}", generate_uuid())
}
