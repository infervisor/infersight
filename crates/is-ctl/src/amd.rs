//! AMD GPU control operations via AMD SMI (is-amd-ffi).

use anyhow::{Result, bail};
use colored::Colorize;

use crate::{format_bytes, GpuListEntry, OutputFormat, PerfLevel};

/// Initialize AMD SMI or fail with a clear message.
fn init_amd() -> Result<()> {
    let ret = is_amd_ffi::amd_smi_init();
    if ret != 0 {
        bail!(
            "Failed to initialize AMD SMI (error code: {ret}).\n\
             Ensure AMD GPU drivers and ROCm are installed.\n\
             Hint: Run 'rocm-smi' to check driver status."
        );
    }
    Ok(())
}

// ─── LIST ────────────────────────────────────────────────────────────────────

pub fn list_gpus(format: &OutputFormat) -> Result<()> {
    init_amd()?;
    let count = is_amd_ffi::amd_smi_get_device_count();

    if count == 0 {
        bail!("No AMD GPUs detected");
    }

    let mut entries = Vec::new();

    for i in 0..count {
        let raw = is_amd_ffi::amd_smi_collect_device(i);
        entries.push(GpuListEntry {
            index: i,
            name: raw.brand.clone(),
            uuid: raw.uuid.clone(),
            temperature_c: raw.temperature_celsius,
            power_w: raw.power_usage_mw / 1000,
            power_limit_w: raw.power_limit_mw / 1000,
            memory_used: if raw.memory_used_bytes > 0 {
                format_bytes(raw.memory_used_bytes)
            } else {
                "N/A".into()
            },
            memory_total: if raw.memory_total_bytes > 0 {
                format_bytes(raw.memory_total_bytes)
            } else {
                "N/A".into()
            },
        });
    }

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&entries)?);
        }
        OutputFormat::Text => {
            println!("{}", format!("AMD GPUs detected: {count}").green());
            println!("{}", "─".repeat(80));
            println!(
                "{:>3} {:30} {:>6} {:>12} {:>16}",
                "ID".bold(),
                "Name".bold(),
                "Temp".bold(),
                "Power".bold(),
                "Memory".bold()
            );
            println!("{}", "─".repeat(80));
            for e in &entries {
                println!(
                    "{:>3} {:30} {:>4}°C {:>5}/{:<5}W {:>16}",
                    e.index.to_string().cyan(),
                    e.name,
                    e.temperature_c,
                    e.power_w,
                    e.power_limit_w,
                    format!("{}/{}", e.memory_used, e.memory_total),
                );
            }
        }
    }

    Ok(())
}

// ─── INFO ────────────────────────────────────────────────────────────────────

pub fn gpu_info(index: u32, format: &OutputFormat) -> Result<()> {
    init_amd()?;
    let count = is_amd_ffi::amd_smi_get_device_count();
    if index >= count {
        bail!("AMD GPU {index} not found (detected {count} devices)");
    }

    let raw = is_amd_ffi::amd_smi_collect_device(index);
    let perf_level = is_amd_ffi::amd_smi_get_perf_level(index);
    let perf_str = match perf_level {
        0 => "auto",
        1 => "low",
        2 => "high",
        3 => "manual",
        _ => "unknown",
    };

    match format {
        OutputFormat::Json => {
            let info = serde_json::json!({
                "index": index,
                "name": raw.brand,
                "uuid": raw.uuid,
                "temperature_c": raw.temperature_celsius,
                "power_usage_mw": raw.power_usage_mw,
                "power_limit_mw": raw.power_limit_mw,
                "memory_used_bytes": raw.memory_used_bytes,
                "memory_total_bytes": raw.memory_total_bytes,
                "clock_core_mhz": raw.clock_core_mhz,
                "clock_memory_mhz": raw.clock_memory_mhz,
                "gpu_utilization_percent": raw.gpu_utilization_percent,
                "memory_utilization_percent": raw.memory_utilization_percent,
                "fan_speed_rpm": raw.fan_speed_rpm,
                "perf_level": perf_str,
            });
            println!("{}", serde_json::to_string_pretty(&info)?);
        }
        OutputFormat::Text => {
            println!("{}", format!("GPU {index}: {}", raw.brand).cyan().bold());
            println!("{}", "─".repeat(50));
            println!("  UUID:          {}", raw.uuid);
            println!("  Temperature:   {}°C", raw.temperature_celsius);
            println!(
                "  Power:         {} / {} W",
                raw.power_usage_mw / 1000,
                raw.power_limit_mw / 1000
            );
            if raw.memory_total_bytes > 0 {
                println!(
                    "  Memory:        {} / {}",
                    format_bytes(raw.memory_used_bytes),
                    format_bytes(raw.memory_total_bytes)
                );
            }
            println!("  Core Clock:    {} MHz", raw.clock_core_mhz);
            println!("  Memory Clock:  {} MHz", raw.clock_memory_mhz);
            println!("  GPU Util:      {}%", raw.gpu_utilization_percent);
            println!("  Mem Util:      {}%", raw.memory_utilization_percent);
            if raw.fan_speed_rpm > 0 {
                println!("  Fan:           {} RPM", raw.fan_speed_rpm);
            }
            println!("  Perf Level:    {perf_str}");
        }
    }

    Ok(())
}

// ─── SET POWER LIMIT ─────────────────────────────────────────────────────────

pub fn set_power_limit(index: u32, watts: u32) -> Result<()> {
    init_amd()?;
    let count = is_amd_ffi::amd_smi_get_device_count();
    if index >= count {
        bail!("AMD GPU {index} not found (detected {count} devices)");
    }

    let milliwatts = watts as u64 * 1000;
    let ret = is_amd_ffi::amd_smi_set_power_limit(index, milliwatts);
    if ret != 0 {
        bail!(
            "Failed to set power limit for AMD GPU {index} (error code: {ret}).\n\
             Do you have root/sudo permissions?"
        );
    }

    println!(
        "{}",
        format!("✓ AMD GPU {index}: Power limit set to {watts}W").green()
    );
    Ok(())
}

// ─── SET PERF LEVEL ──────────────────────────────────────────────────────────

pub fn set_perf(index: u32, level: &PerfLevel) -> Result<()> {
    init_amd()?;
    let count = is_amd_ffi::amd_smi_get_device_count();
    if index >= count {
        bail!("AMD GPU {index} not found (detected {count} devices)");
    }

    let level_str = match level {
        PerfLevel::Auto => "auto",
        PerfLevel::Low => "low",
        PerfLevel::High => "high",
    };

    let success = {
        #[cfg(all(feature = "amd", target_arch = "x86_64"))]
        {
            let cxx_str = cxx::let_cxx_string!(s = level_str);
            is_amd_ffi::amd_smi_set_perf_level(index, &s)
        }
        #[cfg(not(all(feature = "amd", target_arch = "x86_64")))]
        {
            is_amd_ffi::amd_smi_set_perf_level(index, level_str)
        }
    };

    if !success {
        bail!(
            "Failed to set performance level for AMD GPU {index}.\n\
             Do you have root/sudo permissions?"
        );
    }

    println!(
        "{}",
        format!("✓ AMD GPU {index}: Performance set to {}", level_str.to_uppercase()).green()
    );
    Ok(())
}

pub fn set_perf_all(level: &PerfLevel) -> Result<()> {
    init_amd()?;
    let count = is_amd_ffi::amd_smi_get_device_count();
    for i in 0..count {
        if let Err(e) = set_perf(i, level) {
            eprintln!("{}", format!("✗ AMD GPU {i}: {e}").red());
        }
    }
    Ok(())
}

// ─── RESET ───────────────────────────────────────────────────────────────────

pub fn reset_clocks(index: u32) -> Result<()> {
    init_amd()?;
    let count = is_amd_ffi::amd_smi_get_device_count();
    if index >= count {
        bail!("AMD GPU {index} not found (detected {count} devices)");
    }

    // Reset by setting perf level to auto
    let success = {
        #[cfg(all(feature = "amd", target_arch = "x86_64"))]
        {
            let cxx_str = cxx::let_cxx_string!(s = "auto");
            is_amd_ffi::amd_smi_set_perf_level(index, &s)
        }
        #[cfg(not(all(feature = "amd", target_arch = "x86_64")))]
        {
            is_amd_ffi::amd_smi_set_perf_level(index, "auto")
        }
    };

    if !success {
        bail!(
            "Failed to reset AMD GPU {index}. Do you have root/sudo permissions?"
        );
    }

    println!(
        "{}",
        format!("✓ AMD GPU {index}: Reset to defaults (auto perf level)").green()
    );
    Ok(())
}

pub fn reset_all() -> Result<()> {
    init_amd()?;
    let count = is_amd_ffi::amd_smi_get_device_count();
    for i in 0..count {
        if let Err(e) = reset_clocks(i) {
            eprintln!("{}", format!("✗ AMD GPU {i}: {e}").red());
        }
    }
    println!("{}", "✓ All AMD GPUs reset to defaults".green());
    Ok(())
}
