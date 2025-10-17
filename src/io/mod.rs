// src/io/mod.rs - Input/Output operations module
pub mod reader;
pub mod writer;
pub mod watcher;

// Re-exports for convenience
pub use reader::FileReader;
pub use writer::FileWriter;
pub use watcher::FileWatcher;