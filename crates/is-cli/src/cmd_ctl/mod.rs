//! Ctl subcommand — GPU power & clock control.
//! Delegates entirely to the `is-ctl` library (DRY principle).

use anyhow::Result;
use crate::CtlCommands;

pub fn run(cmd: CtlCommands) -> Result<()> {
    match cmd {
        CtlCommands::NvidiaList { format } => {
            #[cfg(feature = "nvidia")]
            { is_ctl::nvidia::list_gpus(&format) }
            #[cfg(not(feature = "nvidia"))]
            { let _ = format; anyhow::bail!("NVIDIA support not compiled") }
        }
        CtlCommands::NvidiaInfo { gpu, format } => {
            #[cfg(feature = "nvidia")]
            { is_ctl::nvidia::gpu_info(gpu, &format) }
            #[cfg(not(feature = "nvidia"))]
            { let _ = (gpu, format); anyhow::bail!("NVIDIA support not compiled") }
        }
        CtlCommands::NvidiaSetClocks { gpu, mem, graphics, all } => {
            #[cfg(feature = "nvidia")]
            {
                if all {
                    is_ctl::nvidia::set_clocks_all(mem, graphics)
                } else {
                    let idx = gpu.ok_or_else(|| anyhow::anyhow!("Specify GPU index or use --all"))?;
                    is_ctl::nvidia::set_clocks(idx, mem, graphics)
                }
            }
            #[cfg(not(feature = "nvidia"))]
            { let _ = (gpu, mem, graphics, all); anyhow::bail!("NVIDIA support not compiled") }
        }
        CtlCommands::NvidiaSetPowerLimit { gpu, watts } => {
            #[cfg(feature = "nvidia")]
            { is_ctl::nvidia::set_power_limit(gpu, watts) }
            #[cfg(not(feature = "nvidia"))]
            { let _ = (gpu, watts); anyhow::bail!("NVIDIA support not compiled") }
        }
        CtlCommands::NvidiaSetPerf { gpu, level, all } => {
            #[cfg(feature = "nvidia")]
            {
                if all {
                    is_ctl::nvidia::set_perf_all(&level)
                } else {
                    let idx = gpu.ok_or_else(|| anyhow::anyhow!("Specify GPU index or use --all"))?;
                    is_ctl::nvidia::set_perf(idx, &level)
                }
            }
            #[cfg(not(feature = "nvidia"))]
            { let _ = (gpu, level, all); anyhow::bail!("NVIDIA support not compiled") }
        }
        CtlCommands::NvidiaReset { gpu, all } => {
            #[cfg(feature = "nvidia")]
            {
                if all {
                    is_ctl::nvidia::reset_all()
                } else {
                    let idx = gpu.ok_or_else(|| anyhow::anyhow!("Specify GPU index or use --all"))?;
                    is_ctl::nvidia::reset_clocks(idx)
                }
            }
            #[cfg(not(feature = "nvidia"))]
            { let _ = (gpu, all); anyhow::bail!("NVIDIA support not compiled") }
        }
        CtlCommands::NvidiaSupportedClocks { gpu, format } => {
            #[cfg(feature = "nvidia")]
            { is_ctl::nvidia::supported_clocks(gpu, &format) }
            #[cfg(not(feature = "nvidia"))]
            { let _ = (gpu, format); anyhow::bail!("NVIDIA support not compiled") }
        }
        // AMD commands
        CtlCommands::AmdList { format } => {
            is_ctl::amd::list_gpus(&format)
        }
        CtlCommands::AmdInfo { gpu, format } => {
            is_ctl::amd::gpu_info(gpu, &format)
        }
        CtlCommands::AmdSetPowerLimit { gpu, watts } => {
            is_ctl::amd::set_power_limit(gpu, watts)
        }
        CtlCommands::AmdSetPerf { gpu, level, all } => {
            if all {
                is_ctl::amd::set_perf_all(&level)
            } else {
                let idx = gpu.ok_or_else(|| anyhow::anyhow!("Specify GPU index or use --all"))?;
                is_ctl::amd::set_perf(idx, &level)
            }
        }
        CtlCommands::AmdReset { gpu, all } => {
            if all {
                is_ctl::amd::reset_all()
            } else {
                let idx = gpu.ok_or_else(|| anyhow::anyhow!("Specify GPU index or use --all"))?;
                is_ctl::amd::reset_clocks(idx)
            }
        }
    }
}
