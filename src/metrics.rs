// src/metrics.rs - Metrics collection and reporting
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[cfg(feature = "metrics-support")]
pub struct Metrics {
    events_processed: AtomicU64,
    events_failed: AtomicU64,
    total_processing_time_ms: AtomicU64,
    last_event_time: AtomicU64,
}

#[cfg(feature = "metrics-support")]
impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            events_processed: AtomicU64::new(0),
            events_failed: AtomicU64::new(0),
            total_processing_time_ms: AtomicU64::new(0),
            last_event_time: AtomicU64::new(0),
        }
    }

    pub fn increment_events_processed(&self) {
        self.events_processed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_events_failed(&self) {
        self.events_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_transformation(&self, duration: Duration) {
        let ms = duration.as_millis() as u64;
        self.total_processing_time_ms
            .fetch_add(ms, Ordering::Relaxed);
        self.last_event_time.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            Ordering::Relaxed,
        );
    }

    pub fn get_stats(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            events_processed: self.events_processed.load(Ordering::Relaxed),
            events_failed: self.events_failed.load(Ordering::Relaxed),
            total_processing_time_ms: self.total_processing_time_ms.load(Ordering::Relaxed),
            last_event_time: self.last_event_time.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub events_processed: u64,
    pub events_failed: u64,
    pub total_processing_time_ms: u64,
    pub last_event_time: u64,
}

impl MetricsSnapshot {
    pub fn average_processing_time_ms(&self) -> f64 {
        if self.events_processed > 0 {
            self.total_processing_time_ms as f64 / self.events_processed as f64
        } else {
            0.0
        }
    }
}

#[cfg(feature = "metrics-support")]
pub async fn start_metrics_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    use metrics_exporter_prometheus::PrometheusBuilder;
    use std::net::SocketAddr;

    let addr: SocketAddr = ([0, 0, 0, 0], port).into();

    PrometheusBuilder::new()
        .with_http_listener(addr)
        .install()?;

    // Keep the server running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}

#[cfg(not(feature = "metrics-support"))]
pub struct Metrics;

#[cfg(not(feature = "metrics-support"))]
impl Metrics {
    pub fn new() -> Self {
        Self
    }

    pub fn increment_events_processed(&self) {}
    pub fn increment_events_failed(&self) {}
    pub fn record_transformation(&self, _duration: Duration) {}
}
