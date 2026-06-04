//! Application-level error types.

use thiserror::Error;

/// Top-level error type for the GPU exporter.
#[derive(Debug, Error)]
pub enum ExporterError {
    #[error("NVIDIA initialization failed: {0}")]
    NvidiaInit(String),

    #[error("NVIDIA device error (index={index}): {message}")]
    NvidiaDevice { index: u32, message: String },

    #[error("AMD SMI initialization failed (code={0})")]
    AmdInit(i32),

    #[error("AMD sysfs error: {0}")]
    AmdSysfs(String),

    #[error("AMD SMI device error (index={index}): {message}")]
    AmdDevice { index: u32, message: String },

    #[error("System metrics collection failed: {0}")]
    System(String),

    #[error("Prometheus registry error: {0}")]
    Registry(#[from] prometheus::Error),

    #[error("HTTP server error: {0}")]
    Http(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

/// Convenience type alias.
pub type Result<T> = std::result::Result<T, ExporterError>;
