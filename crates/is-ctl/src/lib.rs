//! is-ctl — GPU power & clock control library.
//!
//! This crate provides the core control logic for NVIDIA and AMD GPUs.
//! It can be used as a library by other crates (e.g., `is-cli`)
//! or as a standalone binary (`is-ctl`).
//!
//! # Architecture
//! - `nvidia` module: NVIDIA GPU control via NVML (requires `nvidia` feature)
//! - `amd` module: AMD GPU control via AMD SMI (requires `amd` feature or stubs)
//! - Shared types: `OutputFormat`, `PerfLevel`

#[cfg(feature = "nvidia")]
pub mod nvidia;

pub mod amd;

use clap::ValueEnum;
use serde::Serialize;

/// Output format for CLI commands.
#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

/// GPU performance level.
#[derive(Clone, Debug, ValueEnum)]
pub enum PerfLevel {
    Auto,
    Low,
    High,
}

// ─── Shared utilities ────────────────────────────────────────────────────────

/// Format bytes into a human-readable string (GiB, MiB, or KiB).
pub fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GiB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MiB", bytes as f64 / 1_048_576.0)
    } else {
        format!("{:.0} KiB", bytes as f64 / 1024.0)
    }
}

/// Summary entry for a GPU in list output (shared across vendors).
#[derive(Debug, Clone, Serialize)]
pub struct GpuListEntry {
    pub index: u32,
    pub name: String,
    pub uuid: String,
    pub temperature_c: i64,
    pub power_w: u64,
    pub power_limit_w: u64,
    pub memory_used: String,
    pub memory_total: String,
}
