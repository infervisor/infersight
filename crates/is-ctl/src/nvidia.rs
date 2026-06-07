//! NVIDIA GPU control operations via the shared `is-nvidia` crate.

use anyhow::{Context, Result, bail};
use colored::Colorize;
use is_nvidia::NvmlHandle;

use crate::{format_bytes, GpuListEntry, OutputFormat, PerfLevel};

/// Initialize NVML handle or fail with a clear message.
fn init_handle() -> Result<NvmlHandle> {
    NvmlHandle::init().map_err(|e| anyhow::anyhow!(
        "Failed to initialize NVML. Ensure NVIDIA drivers are installed and loaded.\n\
         Hint: Run 'nvidia-smi' to check driver status.\nError: {e}"
    ))
}

// ─── LIST ────────────────────────────────────────────────────────────────────

pub fn list_gpus(format: &OutputFormat) -> Result<()> {
    let handle = init_handle()?;
    let count = handle.device_count();

    if count == 0 {
        bail!("No NVIDIA GPUs detected");
    }

    let mut entries = Vec::new();

    for i in 0..count {
        let metrics = handle.collect_device(i)
            .context(format!("Failed to collect metrics for GPU {i}"))?;

        entries.push(GpuListEntry {
            index: i,
            name: metrics.name,
            uuid: metrics.uuid,
            temperature_c: metrics.temperature_celsius.unwrap_or(0),
            power_w: metrics.power_usage_mw.unwrap_or(0) / 1000,
            power_limit_w: metrics.power_limit_mw.unwrap_or(0) / 1000,
            memory_used: metrics.memory_used_bytes.map(format_bytes).unwrap_or_default(),
            memory_total: metrics.memory_total_bytes.map(format_bytes).unwrap_or_default(),
        });
    }

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&entries)?);
        }
        OutputFormat::Text => {
            let driver = handle.driver_version().unwrap_or_else(|| "N/A".into());
            println!("{}", format!("NVIDIA Driver: {driver}").cyan());
            println!("{}", format!("GPUs detected: {count}").green());
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
    let handle = init_handle()?;
    let metrics = handle.collect_device(index)
        .context(format!("GPU {index} not found"))?;

    let (max_core, max_mem) = handle.max_clocks(index).unwrap_or((0, 0));
    let (min_power, max_power) = handle.power_limit_constraints(index)
        .map(|(min, max)| (Some(min), Some(max)))
        .unwrap_or((None, None));
    let (pcie_gen, pcie_width) = handle.pcie_info(index).unwrap_or((None, None));

    match format {
        OutputFormat::Json => {
            let info = serde_json::json!({
                "index": index,
                "name": metrics.name,
                "uuid": metrics.uuid,
                "temperature_c": metrics.temperature_celsius,
                "power_mw": metrics.power_usage_mw,
                "power_limit_mw": metrics.power_limit_mw,
                "power_min_mw": min_power,
                "power_max_mw": max_power,
                "memory_used_bytes": metrics.memory_used_bytes,
                "memory_total_bytes": metrics.memory_total_bytes,
                "clock_graphics_mhz": metrics.clock_core_mhz,
                "clock_memory_mhz": metrics.clock_memory_mhz,
                "max_clock_graphics_mhz": max_core,
                "max_clock_memory_mhz": max_mem,
                "fan_speed_percent": metrics.fan_speed_percent,
                "pcie_gen": pcie_gen,
                "pcie_width": pcie_width,
            });
            println!("{}", serde_json::to_string_pretty(&info)?);
        }
        OutputFormat::Text => {
            println!("{}", format!("GPU {index}: {}", metrics.name).cyan().bold());
            println!("{}", "─".repeat(50));
            println!("  UUID:          {}", metrics.uuid);
            println!("  Temperature:   {}°C", metrics.temperature_celsius.unwrap_or(0));
            println!(
                "  Power:         {} / {} W",
                metrics.power_usage_mw.unwrap_or(0) / 1000,
                metrics.power_limit_mw.unwrap_or(0) / 1000
            );
            if let (Some(min), Some(max)) = (min_power, max_power) {
                println!("  Power Range:   {} - {} W", min / 1000, max / 1000);
            }
            if let (Some(used), Some(total)) = (metrics.memory_used_bytes, metrics.memory_total_bytes) {
                println!(
                    "  Memory:        {} / {}",
                    format_bytes(used),
                    format_bytes(total)
                );
            }
            println!("  Core Clock:    {} MHz (max: {max_core} MHz)", metrics.clock_core_mhz.unwrap_or(0));
            println!("  Memory Clock:  {} MHz (max: {max_mem} MHz)", metrics.clock_memory_mhz.unwrap_or(0));
            if let Some(f) = metrics.fan_speed_percent {
                println!("  Fan:           {f}%");
            }
            if let (Some(gen), Some(width)) = (pcie_gen, pcie_width) {
                println!("  PCIe:          Gen{gen} x{width}");
            }
        }
    }

    Ok(())
}

// ─── SET CLOCKS ──────────────────────────────────────────────────────────────

pub fn set_clocks(index: u32, mem_clk: u32, graphics_clk: u32) -> Result<()> {
    let handle = init_handle()?;

    // Validate clocks
    let supported_mem = handle.supported_memory_clocks(index)
        .context("Failed to query supported memory clocks")?;

    if !supported_mem.contains(&mem_clk) {
        bail!(
            "Memory clock {mem_clk} MHz not supported.\n\
             Supported: {supported_mem:?}\n\
             Hint: Use 'is-ctl supported-clocks {index}' to see valid combinations."
        );
    }

    let supported_gfx = handle.supported_graphics_clocks(index, mem_clk)
        .context("Failed to query supported graphics clocks")?;

    if !supported_gfx.contains(&graphics_clk) {
        bail!(
            "Graphics clock {graphics_clk} MHz not supported for mem={mem_clk} MHz.\n\
             Supported range: {} - {} MHz",
            supported_gfx.last().unwrap_or(&0),
            supported_gfx.first().unwrap_or(&0),
        );
    }

    handle.set_applications_clocks(index, mem_clk, graphics_clk)
        .context("Failed to set application clocks. Do you have root/sudo permissions?")?;

    println!(
        "{}",
        format!(
            "✓ GPU {index}: Set clocks to mem={mem_clk} MHz, graphics={graphics_clk} MHz"
        )
        .green()
    );

    Ok(())
}

pub fn set_clocks_all(mem_clk: u32, graphics_clk: u32) -> Result<()> {
    let handle = init_handle()?;
    let count = handle.device_count();

    for i in 0..count {
        if let Err(e) = set_clocks_single(&handle, i, mem_clk, graphics_clk) {
            eprintln!("{}", format!("✗ GPU {i}: {e}").red());
        }
    }
    Ok(())
}

fn set_clocks_single(handle: &NvmlHandle, index: u32, mem_clk: u32, graphics_clk: u32) -> Result<()> {
    let supported_mem = handle.supported_memory_clocks(index)?;
    if !supported_mem.contains(&mem_clk) {
        bail!("Memory clock {mem_clk} MHz not supported");
    }

    let supported_gfx = handle.supported_graphics_clocks(index, mem_clk)?;
    if !supported_gfx.contains(&graphics_clk) {
        bail!("Graphics clock {graphics_clk} MHz not supported");
    }

    handle.set_applications_clocks(index, mem_clk, graphics_clk)?;
    println!(
        "{}",
        format!("✓ GPU {index}: Set clocks to mem={mem_clk}, graphics={graphics_clk} MHz").green()
    );
    Ok(())
}

// ─── SET POWER LIMIT ─────────────────────────────────────────────────────────

pub fn set_power_limit(index: u32, watts: u32) -> Result<()> {
    let handle = init_handle()?;
    let milliwatts = watts * 1000;

    // Check constraints
    if let Ok((min, max)) = handle.power_limit_constraints(index) {
        if milliwatts < min || milliwatts > max {
            bail!(
                "Power limit {watts}W out of range. Valid: {} - {} W",
                min / 1000,
                max / 1000
            );
        }
    }

    handle.set_power_limit(index, milliwatts)
        .context("Failed to set power limit. Do you have root/sudo permissions?")?;

    println!(
        "{}",
        format!("✓ GPU {index}: Power limit set to {watts}W").green()
    );

    Ok(())
}

// ─── SET PERF LEVEL ──────────────────────────────────────────────────────────

pub fn set_perf(index: u32, level: &PerfLevel) -> Result<()> {
    let handle = init_handle()?;

    match level {
        PerfLevel::Auto => {
            handle.reset_applications_clocks(index)
                .context("Failed to reset to auto. Need root?")?;
            println!("{}", format!("✓ GPU {index}: Performance set to AUTO (default clocks)").green());
        }
        PerfLevel::High => {
            let supported_mem = handle.supported_memory_clocks(index)
                .context("Could not determine supported clocks")?;
            let mem_clk = *supported_mem.first()
                .ok_or_else(|| anyhow::anyhow!("No supported memory clocks found"))?;
            let supported_gfx = handle.supported_graphics_clocks(index, mem_clk)
                .context("Could not determine supported graphics clocks")?;
            let gfx_clk = *supported_gfx.first()
                .ok_or_else(|| anyhow::anyhow!("No supported graphics clocks found"))?;

            handle.set_applications_clocks(index, mem_clk, gfx_clk)
                .context("Failed to set high perf clocks. Need root?")?;
            println!(
                "{}",
                format!("✓ GPU {index}: Performance set to HIGH (mem={mem_clk}, gfx={gfx_clk} MHz)").green()
            );
        }
        PerfLevel::Low => {
            let supported_mem = handle.supported_memory_clocks(index)
                .context("Could not determine supported clocks")?;
            let mem_clk = *supported_mem.last()
                .ok_or_else(|| anyhow::anyhow!("No supported memory clocks found"))?;
            let supported_gfx = handle.supported_graphics_clocks(index, mem_clk)
                .context("Could not determine supported graphics clocks")?;
            let gfx_clk = *supported_gfx.last()
                .ok_or_else(|| anyhow::anyhow!("No supported graphics clocks found"))?;

            handle.set_applications_clocks(index, mem_clk, gfx_clk)
                .context("Failed to set low perf clocks. Need root?")?;
            println!(
                "{}",
                format!("✓ GPU {index}: Performance set to LOW (mem={mem_clk}, gfx={gfx_clk} MHz)").green()
            );
        }
    }

    Ok(())
}

pub fn set_perf_all(level: &PerfLevel) -> Result<()> {
    let handle = init_handle()?;
    let count = handle.device_count();
    for i in 0..count {
        if let Err(e) = set_perf(i, level) {
            eprintln!("{}", format!("✗ GPU {i}: {e}").red());
        }
    }
    Ok(())
}

// ─── RESET ───────────────────────────────────────────────────────────────────

pub fn reset_clocks(index: u32) -> Result<()> {
    let handle = init_handle()?;

    handle.reset_applications_clocks(index)
        .context("Failed to reset clocks. Do you have root/sudo permissions?")?;

    println!(
        "{}",
        format!("✓ GPU {index}: Clocks reset to default").green()
    );

    Ok(())
}

pub fn reset_all() -> Result<()> {
    let handle = init_handle()?;
    let count = handle.device_count();

    for i in 0..count {
        if let Err(e) = reset_clocks(i) {
            eprintln!("{}", format!("✗ GPU {i}: {e}").red());
        }
    }

    println!("{}", "✓ All GPUs reset to defaults".green());
    Ok(())
}

// ─── SUPPORTED CLOCKS ────────────────────────────────────────────────────────

pub fn supported_clocks(index: u32, format: &OutputFormat) -> Result<()> {
    let handle = init_handle()?;

    let supported_mem = handle.supported_memory_clocks(index)
        .context("Failed to query supported memory clocks")?;

    // Get device name for display
    let device_name = handle.collect_device(index)
        .map(|m| m.name)
        .unwrap_or_else(|_| "Unknown".into());

    match format {
        OutputFormat::Json => {
            let mut clock_map = Vec::new();
            for &mem in &supported_mem {
                let gfx = handle.supported_graphics_clocks(index, mem).unwrap_or_default();
                clock_map.push(serde_json::json!({
                    "memory_mhz": mem,
                    "graphics_mhz_range": [gfx.last(), gfx.first()],
                    "graphics_count": gfx.len(),
                }));
            }
            println!("{}", serde_json::to_string_pretty(&clock_map)?);
        }
        OutputFormat::Text => {
            println!("{}", format!("GPU {index}: {device_name} — Supported Clocks").cyan().bold());
            println!("{}", "─".repeat(60));
            println!(
                "{:>12} {:>15} {:>15} {:>8}",
                "Mem (MHz)".bold(),
                "GFX Min".bold(),
                "GFX Max".bold(),
                "Steps".bold()
            );
            println!("{}", "─".repeat(60));

            for &mem in &supported_mem {
                let gfx = handle.supported_graphics_clocks(index, mem).unwrap_or_default();
                let min = gfx.last().copied().unwrap_or(0);
                let max = gfx.first().copied().unwrap_or(0);
                println!(
                    "{:>12} {:>15} {:>15} {:>8}",
                    mem, min, max, gfx.len()
                );
            }
        }
    }

    Ok(())
}
