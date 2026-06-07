//! CLI configuration and application settings.

use clap::Parser;

/// GPU Metrics Exporter — Prometheus exporter for AMD, NVIDIA, and system metrics.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "is-exporter",
    version,
    about = "Professional GPU metrics exporter for Prometheus",
    long_about = "Collects GPU and system metrics from AMD (via ROCm SMI), NVIDIA (via NVML), \
                  and system sources, exposing them as Prometheus metrics over HTTP."
)]
pub struct Config {
    /// Port for the Prometheus metrics HTTP endpoint.
    #[arg(short, long, default_value_t = 9835)]
    pub port: u16,

    /// Metrics collection interval in seconds.
    #[arg(short, long, default_value_t = 5)]
    pub interval: u64,

    /// Enable NVIDIA GPU metrics collection.
    #[arg(long)]
    pub nvidia: bool,

    /// Enable AMD GPU metrics collection.
    #[arg(long)]
    pub amd: bool,

    /// Enable system metrics collection (CPU, memory, disk, network).
    #[arg(long)]
    pub system: bool,

    /// Enable Google Cloud TPU metrics collection.
    #[arg(long)]
    pub tpu: bool,

    /// Enable all collectors (equivalent to --nvidia --amd --system --tpu).
    #[arg(long)]
    pub all: bool,

    /// Bind address for the HTTP server.
    #[arg(long, default_value = "0.0.0.0")]
    pub bind: String,

    /// Log level filter (e.g., info, debug, trace, warn, error).
    #[arg(long, default_value = "info")]
    pub log_level: String,
}

impl Config {
    /// Returns true if NVIDIA collection is enabled (either explicitly or via --all).
    pub fn nvidia_enabled(&self) -> bool {
        self.nvidia || self.all
    }

    /// Returns true if AMD collection is enabled (either explicitly or via --all).
    pub fn amd_enabled(&self) -> bool {
        self.amd || self.all
    }

    /// Returns true if system collection is enabled (either explicitly or via --all).
    pub fn system_enabled(&self) -> bool {
        self.system || self.all
    }

    /// Returns true if TPU collection is enabled (either explicitly or via --all).
    pub fn tpu_enabled(&self) -> bool {
        self.tpu || self.all
    }

    /// Returns true if no collectors are explicitly enabled.
    pub fn no_collectors_enabled(&self) -> bool {
        !self.nvidia_enabled() && !self.amd_enabled() && !self.system_enabled() && !self.tpu_enabled()
    }
}
