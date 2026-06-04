//! NVIDIA GPU operations via NVML (real implementation).

use nvml_wrapper::enum_wrappers::device::{Clock, ClockId, TemperatureSensor};
use nvml_wrapper::enums::device::UsedGpuMemory;
use nvml_wrapper::Nvml;

use crate::{GpuProcessInfo, NvidiaDeviceMetrics, NvidiaError, Result};

/// Extract bytes from UsedGpuMemory enum.
fn gpu_mem_bytes(mem: &UsedGpuMemory) -> u64 {
    match mem {
        UsedGpuMemory::Used(bytes) => *bytes,
        UsedGpuMemory::Unavailable => 0,
    }
}

/// Handle wrapping an initialized NVML instance.
///
/// This is the primary entry point for all NVIDIA GPU operations.
/// It provides metrics collection and control operations.
pub struct NvmlHandle {
    nvml: Nvml,
    device_count: u32,
}

impl NvmlHandle {
    /// Initialize NVML and detect available NVIDIA GPUs.
    pub fn init() -> Result<Self> {
        let nvml = Nvml::init().map_err(|e| NvidiaError::Init(format!("{e:?}")))?;

        let device_count = nvml
            .device_count()
            .map_err(|e| NvidiaError::Init(format!("Failed to get device count: {e:?}")))?;

        Ok(Self { nvml, device_count })
    }

    /// Get the number of detected NVIDIA GPUs.
    pub fn device_count(&self) -> u32 {
        self.device_count
    }

    /// Get the NVIDIA driver version string.
    pub fn driver_version(&self) -> Option<String> {
        self.nvml.sys_driver_version().ok()
    }

    /// Get the CUDA version string (e.g., "12.4").
    pub fn cuda_version(&self) -> Option<String> {
        self.nvml
            .sys_cuda_driver_version()
            .ok()
            .map(|v| format!("{}.{}", v / 1000, (v % 1000) / 10))
    }

    /// Collect metrics for a single device by index.
    pub fn collect_device(&self, index: u32) -> Result<NvidiaDeviceMetrics> {
        let device = self
            .nvml
            .device_by_index(index)
            .map_err(|_| NvidiaError::DeviceNotFound(index))?;

        let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        let uuid = device
            .uuid()
            .unwrap_or_else(|_| format!("nvidia-gpu-{index}"));

        let mut metrics = NvidiaDeviceMetrics {
            index,
            name,
            uuid,
            gpu_utilization_percent: None,
            memory_utilization_percent: None,
            memory_total_bytes: None,
            memory_used_bytes: None,
            memory_free_bytes: None,
            power_usage_mw: None,
            power_limit_mw: None,
            clock_core_mhz: None,
            clock_memory_mhz: None,
            temperature_celsius: None,
            fan_speed_percent: None,
        };

        // GPU & Memory utilization
        if let Ok(util) = device.utilization_rates() {
            metrics.gpu_utilization_percent = Some(util.gpu as i64);
            metrics.memory_utilization_percent = Some(util.memory as i64);
        }

        // Power usage (NVML reports in milliwatts)
        if let Ok(power) = device.power_usage() {
            metrics.power_usage_mw = Some(power as u64);
        }
        if let Ok(limit) = device.enforced_power_limit() {
            metrics.power_limit_mw = Some(limit as u64);
        }

        // Clock speeds
        if let Ok(clock) = device.clock(Clock::Graphics, ClockId::Current) {
            metrics.clock_core_mhz = Some(clock);
        }
        if let Ok(clock) = device.clock(Clock::Memory, ClockId::Current) {
            metrics.clock_memory_mhz = Some(clock);
        }

        // Temperature
        if let Ok(temp) = device.temperature(TemperatureSensor::Gpu) {
            metrics.temperature_celsius = Some(temp as i64);
        }

        // Fan speed (percentage)
        if let Ok(fan) = device.fan_speed(0) {
            metrics.fan_speed_percent = Some(fan);
        }

        // Memory info
        if let Ok(mem) = device.memory_info() {
            metrics.memory_total_bytes = Some(mem.total);
            metrics.memory_used_bytes = Some(mem.used);
            metrics.memory_free_bytes = Some(mem.free);
        }

        Ok(metrics)
    }

    /// Collect metrics for all detected devices.
    pub fn collect_all(&self) -> Result<Vec<NvidiaDeviceMetrics>> {
        let mut results = Vec::with_capacity(self.device_count as usize);
        for i in 0..self.device_count {
            match self.collect_device(i) {
                Ok(m) => results.push(m),
                Err(_) => continue,
            }
        }
        Ok(results)
    }

    // ─── Control operations ──────────────────────────────────────────────────

    /// Get supported memory clocks for a device.
    pub fn supported_memory_clocks(&self, index: u32) -> Result<Vec<u32>> {
        let device = self
            .nvml
            .device_by_index(index)
            .map_err(|_| NvidiaError::DeviceNotFound(index))?;
        device
            .supported_memory_clocks()
            .map_err(|e| NvidiaError::Operation(format!("Failed to query memory clocks: {e:?}")))
    }

    /// Get supported graphics clocks for a given memory clock.
    pub fn supported_graphics_clocks(&self, index: u32, mem_clock: u32) -> Result<Vec<u32>> {
        let device = self
            .nvml
            .device_by_index(index)
            .map_err(|_| NvidiaError::DeviceNotFound(index))?;
        device
            .supported_graphics_clocks(mem_clock)
            .map_err(|e| NvidiaError::Operation(format!("Failed to query graphics clocks: {e:?}")))
    }

    /// Set application clocks (memory and graphics) for a device.
    /// Requires root/sudo.
    pub fn set_applications_clocks(
        &self,
        index: u32,
        mem_clk: u32,
        graphics_clk: u32,
    ) -> Result<()> {
        let mut device = self
            .nvml
            .device_by_index(index)
            .map_err(|_| NvidiaError::DeviceNotFound(index))?;
        device
            .set_applications_clocks(mem_clk, graphics_clk)
            .map_err(|e| NvidiaError::Operation(format!("Failed to set clocks: {e:?}")))
    }

    /// Reset application clocks to default for a device.
    /// Requires root/sudo.
    pub fn reset_applications_clocks(&self, index: u32) -> Result<()> {
        let mut device = self
            .nvml
            .device_by_index(index)
            .map_err(|_| NvidiaError::DeviceNotFound(index))?;
        device
            .reset_applications_clocks()
            .map_err(|e| NvidiaError::Operation(format!("Failed to reset clocks: {e:?}")))
    }

    /// Set power management limit in milliwatts for a device.
    /// Requires root/sudo.
    pub fn set_power_limit(&self, index: u32, milliwatts: u32) -> Result<()> {
        let mut device = self
            .nvml
            .device_by_index(index)
            .map_err(|_| NvidiaError::DeviceNotFound(index))?;
        device
            .set_power_management_limit(milliwatts)
            .map_err(|e| NvidiaError::Operation(format!("Failed to set power limit: {e:?}")))
    }

    /// Get power management limit constraints (min, max) in milliwatts.
    pub fn power_limit_constraints(&self, index: u32) -> Result<(u32, u32)> {
        let device = self
            .nvml
            .device_by_index(index)
            .map_err(|_| NvidiaError::DeviceNotFound(index))?;
        let constraints = device
            .power_management_limit_constraints()
            .map_err(|e| NvidiaError::Operation(format!("Failed to get power constraints: {e:?}")))?;
        Ok((constraints.min_limit, constraints.max_limit))
    }

    /// Get max clock info for graphics and memory.
    pub fn max_clocks(&self, index: u32) -> Result<(u32, u32)> {
        let device = self
            .nvml
            .device_by_index(index)
            .map_err(|_| NvidiaError::DeviceNotFound(index))?;
        let max_gfx = device.max_clock_info(Clock::Graphics).unwrap_or(0);
        let max_mem = device.max_clock_info(Clock::Memory).unwrap_or(0);
        Ok((max_gfx, max_mem))
    }

    /// Get PCIe info (generation, width) for a device.
    pub fn pcie_info(&self, index: u32) -> Result<(Option<u32>, Option<u32>)> {
        let device = self
            .nvml
            .device_by_index(index)
            .map_err(|_| NvidiaError::DeviceNotFound(index))?;
        Ok((
            device.current_pcie_link_gen().ok(),
            device.current_pcie_link_width().ok(),
        ))
    }

    /// Collect all processes running on all GPUs.
    pub fn collect_processes(&self) -> Vec<GpuProcessInfo> {
        let mut processes = Vec::new();

        for idx in 0..self.device_count {
            let device = match self.nvml.device_by_index(idx) {
                Ok(d) => d,
                Err(_) => continue,
            };

            // Compute processes
            if let Ok(compute_procs) = device.running_compute_processes() {
                for proc in compute_procs {
                    let mem = gpu_mem_bytes(&proc.used_gpu_memory);
                    if let Some(existing) = processes.iter_mut().find(|p: &&mut GpuProcessInfo| {
                        p.pid == proc.pid && p.gpu_index == idx
                    }) {
                        existing.process_type = "C+G".to_string();
                        existing.gpu_memory_bytes = existing.gpu_memory_bytes.max(mem);
                    } else {
                        processes.push(GpuProcessInfo {
                            pid: proc.pid,
                            gpu_index: idx,
                            gpu_memory_bytes: mem,
                            process_type: "C".to_string(),
                        });
                    }
                }
            }

            // Graphics processes
            if let Ok(graphics_procs) = device.running_graphics_processes() {
                for proc in graphics_procs {
                    let mem = gpu_mem_bytes(&proc.used_gpu_memory);
                    if let Some(existing) = processes.iter_mut().find(|p: &&mut GpuProcessInfo| {
                        p.pid == proc.pid && p.gpu_index == idx
                    }) {
                        existing.process_type = "C+G".to_string();
                        existing.gpu_memory_bytes = existing.gpu_memory_bytes.max(mem);
                    } else {
                        processes.push(GpuProcessInfo {
                            pid: proc.pid,
                            gpu_index: idx,
                            gpu_memory_bytes: mem,
                            process_type: "G".to_string(),
                        });
                    }
                }
            }
        }

        processes
    }
}
