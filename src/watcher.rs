// src/watcher.rs - File watching functionality
#[cfg(feature = "watch-mode")]
use crate::{OcsfTransformer, TransformError, TransformResult};
#[cfg(feature = "watch-mode")]
use notify::{Event, EventKind, RecursiveMode, Result as NotifyResult, Watcher};
#[cfg(feature = "watch-mode")]
use std::path::{Path, PathBuf};
#[cfg(feature = "watch-mode")]
use std::sync::Arc;
#[cfg(feature = "watch-mode")]
use tokio::sync::mpsc;
#[cfg(feature = "watch-mode")]
use tracing::{debug, error, info};

#[cfg(feature = "watch-mode")]
pub struct FileWatcher {
    transformer: Arc<OcsfTransformer>,
    input_path: PathBuf,
    output_path: PathBuf,
    #[allow(dead_code)] // Will be used for polling-based watching in future implementation
    poll_interval: u64,
}

#[cfg(feature = "watch-mode")]
impl FileWatcher {
    pub fn new(
        transformer: OcsfTransformer,
        input_path: PathBuf,
        output_path: PathBuf,
        poll_interval: u64,
    ) -> TransformResult<Self> {
        Ok(Self {
            transformer: Arc::new(transformer),
            input_path,
            output_path,
            poll_interval,
        })
    }

    pub async fn start(self) -> TransformResult<()> {
        info!("Starting file watcher for: {}", self.input_path.display());

        let (tx, mut rx) = mpsc::channel(100);

        // Create watcher
        let mut watcher = notify::recommended_watcher(move |res: NotifyResult<Event>| {
            if let Ok(event) = res {
                let _ = tx.blocking_send(event);
            }
        })
        .map_err(|e| TransformError::WatchError(e.to_string()))?;

        // Watch the path
        watcher
            .watch(&self.input_path, RecursiveMode::Recursive)
            .map_err(|e| TransformError::WatchError(e.to_string()))?;

        info!("Watching for changes...");

        // Process events
        while let Some(event) = rx.recv().await {
            match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) => {
                    for path in event.paths {
                        if self.should_process(&path) {
                            info!("File changed: {}", path.display());
                            self.process_file(&path).await;
                        }
                    }
                }
                EventKind::Remove(_) => {
                    for path in event.paths {
                        debug!("File removed: {}", path.display());
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn should_process(&self, path: &Path) -> bool {
        path.is_file()
            && path
                .extension()
                .is_some_and(|ext| ext == "log" || ext == "txt" || ext == "secure")
    }

    async fn process_file(&self, path: &Path) {
        info!("Processing: {}", path.display());

        match self.transformer.process_file(path).await {
            Ok(events) => {
                let output_file = self.output_path.join(format!(
                    "{}.ocsf.json",
                    path.file_stem().unwrap().to_string_lossy()
                ));

                match tokio::fs::write(&output_file, serde_json::to_string_pretty(&events).unwrap())
                    .await
                {
                    Ok(_) => {
                        info!("Wrote {} events to {}", events.len(), output_file.display());
                    }
                    Err(e) => {
                        error!("Failed to write output: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to process {}: {}", path.display(), e);
            }
        }
    }
}

#[cfg(not(feature = "watch-mode"))]
pub struct FileWatcher;
