use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
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
    histograms: HashMap<String, VecDeque<f64>>,
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
        let values = self.histograms.entry(name.to_string()).or_insert(VecDeque::new());
        if values.len() >= 1000 {
            values.pop_front();
        }
        values.push_back(value);
    }

    pub fn get_counter(&self, name: &str) -> Option<u64> {
        self.counters.get(name).copied()
    }

    pub fn get_gauge(&self, name: &str) -> Option<i64> {
        self.gauges.get(name).copied()
    }

    pub fn get_histogram(&self, name: &str) -> Option<Vec<f64>> {
        self.histograms.get(name).map(|v| v.iter().copied().collect())
    }
}

#[derive(Clone)]
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
    }

    pub fn handle_panic(&self, info: &std::panic::PanicHookInfo) {
        let payload = info.payload();
        let message = if let Some(s) = payload.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        self.logger.error(&format!("Panic occurred: {}", message));
    }
}

pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
