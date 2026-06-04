//! InferSight — Unified GPU management CLI.
//!
//! Single binary providing:
//!   infersight export   — Start Prometheus metrics exporter
//!   infersight top      — Interactive TUI GPU monitor
//!   infersight ctl      — GPU power & clock control (NVIDIA + AMD)

mod cmd_ctl;
mod cmd_export;

use clap::{Parser, Subcommand};
use is_ctl::{OutputFormat, PerfLevel};

#[derive(Parser)]
#[command(
    name = "infersight",
    version,
    about = "InferSight — Professional GPU observability & control toolkit",
    long_about = "Unified binary for GPU monitoring, metrics export, and power control.\n\n\
                  Subcommands:\n\
                  • export  — Start Prometheus metrics HTTP server\n\
                  • top     — Interactive terminal GPU dashboard\n\
                  • ctl     — GPU power/clock control operations (NVIDIA + AMD)"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the Prometheus metrics exporter server
    Export {
        /// Enable NVIDIA GPU metrics
        #[arg(long)]
        nvidia: bool,

        /// Enable AMD GPU metrics
        #[arg(long)]
        amd: bool,

        /// Enable system metrics (CPU, RAM, disk, network)
        #[arg(long)]
        system: bool,

        /// Enable all available collectors
        #[arg(long)]
        all: bool,

        /// HTTP listen port
        #[arg(long, default_value = "9835")]
        port: u16,

        /// HTTP listen address
        #[arg(long, default_value = "0.0.0.0")]
        bind: String,
    },

    /// Interactive terminal GPU monitor (htop for GPUs)
    Top,

    /// GPU power & clock control
    #[command(subcommand)]
    Ctl(CtlCommands),
}

// ─── Ctl command structure (vendor-based) ────────────────────────────────────

#[derive(Subcommand)]
pub enum CtlCommands {
    // ── NVIDIA ──────────────────────────────────────────────────────────────
    /// List all detected NVIDIA GPUs
    #[command(name = "nvidia-list")]
    NvidiaList {
        /// Output format
        #[arg(long, default_value = "text")]
        format: OutputFormat,
    },

    /// Show detailed info for a specific NVIDIA GPU
    #[command(name = "nvidia-info")]
    NvidiaInfo {
        /// GPU index (0-based)
        gpu: u32,
        /// Output format
        #[arg(long, default_value = "text")]
        format: OutputFormat,
    },

    /// Set NVIDIA application clock speeds
    #[command(name = "nvidia-set-clocks")]
    NvidiaSetClocks {
        /// GPU index (0-based), or use --all
        gpu: Option<u32>,
        /// Target memory clock (MHz)
        #[arg(long)]
        mem: u32,
        /// Target graphics/core clock (MHz)
        #[arg(long)]
        graphics: u32,
        /// Apply to all GPUs
        #[arg(long)]
        all: bool,
    },

    /// Set NVIDIA power limit (watts)
    #[command(name = "nvidia-set-power-limit")]
    NvidiaSetPowerLimit {
        /// GPU index (0-based)
        gpu: u32,
        /// Power limit in watts
        #[arg(long)]
        watts: u32,
    },

    /// Set NVIDIA performance level
    #[command(name = "nvidia-set-perf")]
    NvidiaSetPerf {
        /// GPU index (0-based), or use --all
        gpu: Option<u32>,
        /// Performance level: auto, low, high
        #[arg(long)]
        level: PerfLevel,
        /// Apply to all GPUs
        #[arg(long)]
        all: bool,
    },

    /// Reset NVIDIA clocks to default
    #[command(name = "nvidia-reset")]
    NvidiaReset {
        /// GPU index (0-based)
        gpu: Option<u32>,
        /// Reset all GPUs
        #[arg(long)]
        all: bool,
    },

    /// Show NVIDIA supported clock speeds for a GPU
    #[command(name = "nvidia-supported-clocks")]
    NvidiaSupportedClocks {
        /// GPU index (0-based)
        gpu: u32,
        /// Output format
        #[arg(long, default_value = "text")]
        format: OutputFormat,
    },

    // ── AMD ─────────────────────────────────────────────────────────────────
    /// List all detected AMD GPUs
    #[command(name = "amd-list")]
    AmdList {
        /// Output format
        #[arg(long, default_value = "text")]
        format: OutputFormat,
    },

    /// Show detailed info for a specific AMD GPU
    #[command(name = "amd-info")]
    AmdInfo {
        /// GPU index (0-based)
        gpu: u32,
        /// Output format
        #[arg(long, default_value = "text")]
        format: OutputFormat,
    },

    /// Set AMD power limit (watts)
    #[command(name = "amd-set-power-limit")]
    AmdSetPowerLimit {
        /// GPU index (0-based)
        gpu: u32,
        /// Power limit in watts
        #[arg(long)]
        watts: u32,
    },

    /// Set AMD performance level
    #[command(name = "amd-set-perf")]
    AmdSetPerf {
        /// GPU index (0-based), or use --all
        gpu: Option<u32>,
        /// Performance level: auto, low, high
        #[arg(long)]
        level: PerfLevel,
        /// Apply to all GPUs
        #[arg(long)]
        all: bool,
    },

    /// Reset AMD GPU to default settings
    #[command(name = "amd-reset")]
    AmdReset {
        /// GPU index (0-based)
        gpu: Option<u32>,
        /// Reset all GPUs
        #[arg(long)]
        all: bool,
    },
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn".into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Export {
            nvidia,
            amd,
            system,
            all,
            port,
            bind,
        } => cmd_export::run(nvidia, amd, system, all, port, bind),

        Commands::Top => {
            is_top::run().map_err(|e| anyhow::anyhow!("{e}"))
        }

        Commands::Ctl(ctl) => cmd_ctl::run(ctl),
    }
}
