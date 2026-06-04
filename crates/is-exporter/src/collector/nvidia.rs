//! NVIDIA GPU collector using the shared `is-nvidia` crate.

use async_trait::async_trait;
use is_nvidia::{GpuProcessInfo, NvmlHandle};
use tracing::{debug, warn};

use crate::collector::{Collector, GpuSnapshot};
use crate::error::{ExporterError, Result};

/// NVIDIA GPU metrics collector powered by NVML via the shared crate.
pub struct NvidiaCollector {
    handle: Option<NvmlHandle>,
    device_count: u32,
    hostname: String,
}

impl NvidiaCollector {
    /// Create a new uninitialized NVIDIA collector.
    pub fn new() -> Self {
        Self {
            handle: None,
            device_count: 0,
            hostname: whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string()),
        }
    }

    /// Get the NVIDIA driver version string (only available after init).
    pub fn driver_version(&self) -> Option<String> {
        self.handle.as_ref()?.driver_version()
    }

    /// Get the CUDA version string (only available after init).
    pub fn cuda_version(&self) -> Option<String> {
        self.handle.as_ref()?.cuda_version()
    }

    /// Get the device count.
    pub fn device_count(&self) -> u32 {
        self.device_count
    }

    /// Collect GPU processes from NVML.
    pub fn collect_gpu_processes(&self) -> Vec<GpuProcessInfo> {
        match &self.handle {
            Some(handle) => handle.collect_processes(),
            None => Vec::new(),
        }
    }
}

#[async_trait]
impl Collector for NvidiaCollector {
    fn name(&self) -> &'static str {
        "nvidia"
    }

    async fn init(&mut self) -> Result<usize> {
        let handle =
            NvmlHandle::init().map_err(|e| ExporterError::NvidiaInit(format!("{e}")))?;

        let count = handle.device_count();

        if count == 0 {
            return Err(ExporterError::NvidiaInit("No NVIDIA GPUs detected".into()));
        }

        self.device_count = count;
        self.handle = Some(handle);

        Ok(count as usize)
    }

    async fn collect(&self) -> Result<Vec<GpuSnapshot>> {
        let handle = self
            .handle
            .as_ref()
            .ok_or_else(|| ExporterError::NvidiaInit("NVML not initialized".into()))?;

        let mut snapshots = Vec::with_capacity(self.device_count as usize);

        for index in 0..self.device_count {
            let metrics = match handle.collect_device(index) {
                Ok(m) => m,
                Err(e) => {
                    warn!(index, error = %e, "Failed to collect NVIDIA device metrics");
                    continue;
                }
            };

            let mut snapshot = GpuSnapshot::new(
                index,
                "nvidia",
                self.hostname.clone(),
                metrics.name,
                metrics.uuid,
            );

            snapshot.gpu_utilization_percent = metrics.gpu_utilization_percent;
            snapshot.memory_utilization_percent = metrics.memory_utilization_percent;
            snapshot.power_usage_mw = metrics.power_usage_mw;
            snapshot.power_limit_mw = metrics.power_limit_mw;
            snapshot.clock_core_mhz = metrics.clock_core_mhz;
            snapshot.clock_memory_mhz = metrics.clock_memory_mhz;
            snapshot.temperature_celsius = metrics.temperature_celsius;
            snapshot.fan_speed = metrics.fan_speed_percent;
            snapshot.memory_total_bytes = metrics.memory_total_bytes;
            snapshot.memory_used_bytes = metrics.memory_used_bytes;
            snapshot.memory_free_bytes = metrics.memory_free_bytes;

            debug!(index, vendor = "nvidia", "Collected GPU metrics");
            snapshots.push(snapshot);
        }

        Ok(snapshots)
    }
}
