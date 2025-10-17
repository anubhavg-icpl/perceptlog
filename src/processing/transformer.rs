// src/transformer.rs - Core transformation logic
use crate::{
    LogEvent, TransformResult,
    config::TransformerConfig,
    error::TransformError,
    ocsf::OcsfEvent,
    runtime::{VrlRuntime, log_event_to_vrl_value, vrl_value_to_serde_json},
};
use futures::StreamExt;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Main transformer struct that handles log to OCSF transformation
pub struct OcsfTransformer {
    runtime: Arc<RwLock<VrlRuntime>>,
    config: TransformerConfig,
    vrl_script: String,
    #[cfg(feature = "metrics-support")]
    metrics: Arc<crate::metrics::Metrics>,
}

impl OcsfTransformer {
    /// Create a new transformer with the given VRL script file
    pub async fn new(vrl_script_path: impl AsRef<Path>) -> TransformResult<Self> {
        let vrl_script = fs::read_to_string(vrl_script_path.as_ref())
            .map_err(|e| TransformError::IoError(e.to_string()))?;

        Self::with_script(vrl_script).await
    }

    /// Create a new transformer with the given VRL script string
    pub async fn with_script(vrl_script: String) -> TransformResult<Self> {
        let runtime = VrlRuntime::new(&vrl_script)
            .map_err(|e| TransformError::CompileError(e.to_string()))?;

        Ok(Self {
            runtime: Arc::new(RwLock::new(runtime)),
            config: TransformerConfig::default(),
            vrl_script,
            #[cfg(feature = "metrics-support")]
            metrics: Arc::new(crate::metrics::Metrics::new()),
        })
    }

    /// Create a new transformer with configuration
    pub async fn with_config(config: TransformerConfig) -> TransformResult<Self> {
        let vrl_script = fs::read_to_string(&config.script_path)
            .map_err(|e| TransformError::IoError(e.to_string()))?;

        let runtime = VrlRuntime::new(&vrl_script)
            .map_err(|e| TransformError::CompileError(e.to_string()))?;

        Ok(Self {
            runtime: Arc::new(RwLock::new(runtime)),
            config,
            vrl_script,
            #[cfg(feature = "metrics-support")]
            metrics: Arc::new(crate::metrics::Metrics::new()),
        })
    }

    /// Transform a single log line to OCSF format
    pub async fn transform_line(&self, log_line: &str) -> TransformResult<OcsfEvent> {
        let event = LogEvent::new(log_line);
        self.transform_event(event).await
    }

    /// Transform a LogEvent to OCSF format
    pub async fn transform_event(&self, event: LogEvent) -> TransformResult<OcsfEvent> {
        #[cfg(feature = "metrics-support")]
        let start = std::time::Instant::now();

        debug!("Transforming event: {:?}", event.message);

        // Convert LogEvent to VRL Value
        let vrl_value = log_event_to_vrl_value(event);

        // Execute VRL transformation
        let mut runtime = self.runtime.write().await;
        let result = runtime
            .transform(vrl_value)
            .map_err(|e| TransformError::TransformError(e.to_string()))?;

        // Convert VRL result to JSON
        let json_result = vrl_value_to_serde_json(result);

        // Parse into OCSF event
        let ocsf_event: OcsfEvent = serde_json::from_value(json_result)
            .map_err(|e| TransformError::ParseError(e.to_string()))?;

        #[cfg(feature = "metrics-support")]
        {
            self.metrics.record_transformation(start.elapsed());
            self.metrics.increment_events_processed();
        }

        debug!("Successfully transformed event to OCSF format");
        Ok(ocsf_event)
    }

    /// Transform multiple log lines in batch
    pub async fn transform_batch(&self, log_lines: Vec<String>) -> Vec<TransformResult<OcsfEvent>> {
        let mut results = Vec::with_capacity(log_lines.len());

        for line in log_lines {
            results.push(self.transform_line(&line).await);
        }

        results
    }

    /// Process a log file and return transformed OCSF events
    pub async fn process_file(
        &self,
        file_path: impl AsRef<Path>,
    ) -> TransformResult<Vec<OcsfEvent>> {
        let content = fs::read_to_string(file_path.as_ref())
            .map_err(|e| TransformError::IoError(e.to_string()))?;

        let mut events = Vec::new();

        for line in content.lines() {
            if !line.trim().is_empty() {
                match self.transform_line(line).await {
                    Ok(event) => events.push(event),
                    Err(e) => {
                        warn!("Failed to transform line: {} - Error: {}", line, e);
                        if !self.config.skip_errors {
                            return Err(e);
                        }
                    }
                }
            }
        }

        info!("Processed {} events from file", events.len());
        Ok(events)
    }

    /// Stream process a file, yielding OCSF events as they are transformed
    pub async fn stream_file(
        &self,
        file_path: impl AsRef<Path>,
    ) -> TransformResult<impl futures::Stream<Item = TransformResult<OcsfEvent>>> {
        use tokio::fs::File;
        use tokio::io::{AsyncBufReadExt, BufReader};

        use tokio_stream::wrappers::LinesStream;

        let file = File::open(file_path.as_ref())
            .await
            .map_err(|e| TransformError::IoError(e.to_string()))?;

        let reader = BufReader::new(file);
        let lines = reader.lines();
        let stream = LinesStream::new(lines);

        let transformer = self.clone();

        Ok(futures::StreamExt::map(stream, move |line_result| {
            let transformer = transformer.clone();
            async move {
                match line_result {
                    Ok(line) => {
                        if line.trim().is_empty() {
                            Err(TransformError::ParseError("Empty line".to_string()))
                        } else {
                            transformer.transform_line(&line).await
                        }
                    }
                    Err(e) => Err(TransformError::IoError(e.to_string())),
                }
            }
        })
        .buffered(self.config.batch_size))
    }

    /// Reload VRL script (useful for hot-reloading)
    #[cfg(feature = "hot-reload")]
    pub async fn reload_script(&self) -> TransformResult<()> {
        let vrl_script = fs::read_to_string(&self.config.script_path)
            .map_err(|e| TransformError::IoError(e.to_string()))?;

        let new_runtime = VrlRuntime::new(&vrl_script)
            .map_err(|e| TransformError::CompileError(e.to_string()))?;

        let mut runtime = self.runtime.write().await;
        *runtime = new_runtime;

        info!("Successfully reloaded VRL script");
        Ok(())
    }

    /// Validate VRL script without creating a transformer
    pub fn validate_script(vrl_script: &str) -> TransformResult<()> {
        VrlRuntime::new(vrl_script)
            .map(|_| ())
            .map_err(|e| TransformError::CompileError(e.to_string()))
    }

    /// Get metrics (if metrics feature is enabled)
    #[cfg(feature = "metrics-support")]
    pub fn metrics(&self) -> Arc<crate::metrics::Metrics> {
        self.metrics.clone()
    }
}

impl Clone for OcsfTransformer {
    fn clone(&self) -> Self {
        Self {
            runtime: self.runtime.clone(),
            config: self.config.clone(),
            vrl_script: self.vrl_script.clone(),
            #[cfg(feature = "metrics-support")]
            metrics: self.metrics.clone(),
        }
    }
}

