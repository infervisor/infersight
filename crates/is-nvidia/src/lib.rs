//! Shared NVIDIA GPU library via NVML.
//!
//! This crate provides common NVIDIA GPU operations used by multiple crates
//! in the InferSight workspace (exporter, ctl, top).
//!
//! # Usage
//! ```rust,ignore
//! use is_nvidia::{NvmlHandle, NvidiaDeviceMetrics};
//!
//! let handle = NvmlHandle::init().unwrap();
//! let count = handle.device_count();
//! for i in 0..count {
//!     let metrics = handle.collect_device(i).unwrap();
//!     println!("GPU {}: {} — {}°C", i, metrics.name, metrics.temperature_celsius);
//! }
//! ```

#[cfg(feature = "nvidia")]
mod nvml_impl;

#[cfg(feature = "nvidia")]
pub use nvml_impl::*;

// ─── Error types ─────────────────────────────────────────────────────────────

/// Errors from NVIDIA GPU operations.
#[derive(Debug, thiserror::Error)]
pub enum NvidiaError {
    #[error("Failed to initialize NVML: {0}")]
    Init(String),

    #[error("Device not found: GPU {0}")]
    DeviceNotFound(u32),

    #[error("NVML operation failed: {0}")]
    Operation(String),

    #[error("NVIDIA feature not enabled or not supported on this platform")]
    NotSupported,
}

pub type Result<T> = std::result::Result<T, NvidiaError>;

// ─── GPU Process Info ────────────────────────────────────────────────────────

/// Information about a process running on a GPU.
#[derive(Debug, Clone)]
pub struct GpuProcessInfo {
    /// Process ID.
    pub pid: u32,
    /// GPU index the process is running on.
    pub gpu_index: u32,
    /// GPU memory used by this process in bytes.
    pub gpu_memory_bytes: u64,
    /// Process type: "C" (compute), "G" (graphics), or "C+G" (both).
    pub process_type: String,
}

// ─── Device metrics snapshot ─────────────────────────────────────────────────

/// A snapshot of metrics for a single NVIDIA GPU device.
#[derive(Debug, Clone)]
pub struct NvidiaDeviceMetrics {
    /// Device index (0-based).
    pub index: u32,
    /// GPU model name (e.g., "NVIDIA RTX 4090").
    pub name: String,
    /// Unique device identifier (UUID).
    pub uuid: String,
    /// GPU core utilization percentage (0–100).
    pub gpu_utilization_percent: Option<i64>,
    /// Memory controller utilization percentage (0–100).
    pub memory_utilization_percent: Option<i64>,
    /// Total GPU memory in bytes.
    pub memory_total_bytes: Option<u64>,
    /// Used GPU memory in bytes.
    pub memory_used_bytes: Option<u64>,
    /// Free GPU memory in bytes.
    pub memory_free_bytes: Option<u64>,
    /// Current power draw in milliwatts.
    pub power_usage_mw: Option<u64>,
    /// Power limit in milliwatts.
    pub power_limit_mw: Option<u64>,
    /// Core/graphics clock speed in MHz.
    pub clock_core_mhz: Option<u32>,
    /// Memory clock speed in MHz.
    pub clock_memory_mhz: Option<u32>,
    /// GPU temperature in degrees Celsius.
    pub temperature_celsius: Option<i64>,
    /// Fan speed percentage.
    pub fan_speed_percent: Option<u32>,
}

// ─── Stub implementations for non-NVIDIA builds ──────────────────────────────

#[cfg(not(feature = "nvidia"))]
pub struct NvmlHandle;

#[cfg(not(feature = "nvidia"))]
impl NvmlHandle {
    pub fn init() -> Result<Self> {
        Err(NvidiaError::NotSupported)
    }

    pub fn device_count(&self) -> u32 {
        0
    }

    pub fn driver_version(&self) -> Option<String> {
        None
    }

    pub fn cuda_version(&self) -> Option<String> {
        None
    }

    pub fn collect_device(&self, _index: u32) -> Result<NvidiaDeviceMetrics> {
        Err(NvidiaError::NotSupported)
    }

    pub fn collect_all(&self) -> Result<Vec<NvidiaDeviceMetrics>> {
        Err(NvidiaError::NotSupported)
    }
}
