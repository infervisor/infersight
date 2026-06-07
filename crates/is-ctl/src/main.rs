//! is-ctl — GPU power & clock control CLI.
//!
//! Usage:
//!   is-ctl nvidia list                      # List all NVIDIA GPUs
//!   is-ctl nvidia info 0                    # Detailed info for NVIDIA GPU 0
//!   is-ctl nvidia set-clocks 0 --mem 2619 --graphics 1785
//!   is-ctl nvidia set-power-limit 0 --watts 300
//!   is-ctl nvidia set-perf 0 --level high
//!   is-ctl nvidia reset 0                   # Reset NVIDIA GPU 0 to defaults
//!   is-ctl amd list                         # List all AMD GPUs
//!   is-ctl amd info 0                       # Detailed info for AMD GPU 0
//!   is-ctl amd set-power-limit 0 --watts 200
//!   is-ctl amd set-perf 0 --level high
//!   is-ctl amd reset 0

use clap::{Parser, Subcommand};
use is_ctl::{OutputFormat, PerfLevel};

#[derive(Parser)]
#[command(
    name = "is-ctl",
    version,
    about = "GPU power & clock control CLI for NVIDIA and AMD GPUs",
    long_about = "Professional GPU management tool. Set clock speeds, power limits, \
                  and performance levels. Requires root/sudo for most operations."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// NVIDIA GPU control operations
    #[command(subcommand)]
    Nvidia(NvidiaCommands),

    /// AMD GPU control operations
    #[command(subcommand)]
    Amd(AmdCommands),
}

#[derive(Subcommand)]
enum NvidiaCommands {
    /// List all detected NVIDIA GPUs
    List {
        /// Output format
        #[arg(long, default_value = "text")]
        format: OutputFormat,
    },

    /// Show detailed info for a specific GPU
    Info {
        /// GPU index (0-based)
        gpu: u32,
        /// Output format
        #[arg(long, default_value = "text")]
        format: OutputFormat,
    },

    /// Set application clock speeds
    SetClocks {
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

    /// Set power limit (watts)
    SetPowerLimit {
        /// GPU index (0-based)
        gpu: u32,
        /// Power limit in watts
        #[arg(long)]
        watts: u32,
    },

    /// Set performance level
    SetPerf {
        /// GPU index (0-based), or use --all
        gpu: Option<u32>,
        /// Performance level: auto, low, high
        #[arg(long)]
        level: PerfLevel,
        /// Apply to all GPUs
        #[arg(long)]
        all: bool,
    },

    /// Reset clocks to default
    Reset {
        /// GPU index (0-based)
        gpu: Option<u32>,
        /// Reset all GPUs
        #[arg(long)]
        all: bool,
    },

    /// Show supported clock speeds for a GPU
    SupportedClocks {
        /// GPU index (0-based)
        gpu: u32,
        /// Output format
        #[arg(long, default_value = "text")]
        format: OutputFormat,
    },
}

#[derive(Subcommand)]
enum AmdCommands {
    /// List all detected AMD GPUs
    List {
        /// Output format
        #[arg(long, default_value = "text")]
        format: OutputFormat,
    },

    /// Show detailed info for a specific GPU
    Info {
        /// GPU index (0-based)
        gpu: u32,
        /// Output format
        #[arg(long, default_value = "text")]
        format: OutputFormat,
    },

    /// Set power limit (watts)
    SetPowerLimit {
        /// GPU index (0-based)
        gpu: u32,
        /// Power limit in watts
        #[arg(long)]
        watts: u32,
    },

    /// Set performance level
    SetPerf {
        /// GPU index (0-based), or use --all
        gpu: Option<u32>,
        /// Performance level: auto, low, high
        #[arg(long)]
        level: PerfLevel,
        /// Apply to all GPUs
        #[arg(long)]
        all: bool,
    },

    /// Reset to default settings
    Reset {
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
        Commands::Nvidia(cmd) => run_nvidia(cmd),
        Commands::Amd(cmd) => run_amd(cmd),
    }
}

#[cfg(feature = "nvidia")]
fn run_nvidia(cmd: NvidiaCommands) -> anyhow::Result<()> {
    use is_ctl::nvidia;

    match cmd {
        NvidiaCommands::List { format } => nvidia::list_gpus(&format),
        NvidiaCommands::Info { gpu, format } => nvidia::gpu_info(gpu, &format),
        NvidiaCommands::SetClocks { gpu, mem, graphics, all } => {
            if all {
                nvidia::set_clocks_all(mem, graphics)
            } else {
                let idx = gpu.ok_or_else(|| anyhow::anyhow!("Specify GPU index or use --all"))?;
                nvidia::set_clocks(idx, mem, graphics)
            }
        }
        NvidiaCommands::SetPowerLimit { gpu, watts } => nvidia::set_power_limit(gpu, watts),
        NvidiaCommands::SetPerf { gpu, level, all } => {
            if all {
                nvidia::set_perf_all(&level)
            } else {
                let idx = gpu.ok_or_else(|| anyhow::anyhow!("Specify GPU index or use --all"))?;
                nvidia::set_perf(idx, &level)
            }
        }
        NvidiaCommands::Reset { gpu, all } => {
            if all {
                nvidia::reset_all()
            } else {
                let idx = gpu.ok_or_else(|| anyhow::anyhow!("Specify GPU index or use --all"))?;
                nvidia::reset_clocks(idx)
            }
        }
        NvidiaCommands::SupportedClocks { gpu, format } => nvidia::supported_clocks(gpu, &format),
    }
}

#[cfg(not(feature = "nvidia"))]
fn run_nvidia(_cmd: NvidiaCommands) -> anyhow::Result<()> {
    anyhow::bail!("NVIDIA support not compiled. Rebuild with --features nvidia")
}

fn run_amd(cmd: AmdCommands) -> anyhow::Result<()> {
    use is_ctl::amd;

    match cmd {
        AmdCommands::List { format } => amd::list_gpus(&format),
        AmdCommands::Info { gpu, format } => amd::gpu_info(gpu, &format),
        AmdCommands::SetPowerLimit { gpu, watts } => amd::set_power_limit(gpu, watts),
        AmdCommands::SetPerf { gpu, level, all } => {
            if all {
                amd::set_perf_all(&level)
            } else {
                let idx = gpu.ok_or_else(|| anyhow::anyhow!("Specify GPU index or use --all"))?;
                amd::set_perf(idx, &level)
            }
        }
        AmdCommands::Reset { gpu, all } => {
            if all {
                amd::reset_all()
            } else {
                let idx = gpu.ok_or_else(|| anyhow::anyhow!("Specify GPU index or use --all"))?;
                amd::reset_clocks(idx)
            }
        }
    }
}
