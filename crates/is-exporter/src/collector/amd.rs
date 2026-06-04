//! AMD GPU collector using AMD SMI via CXX bridge.

use async_trait::async_trait;
use tracing::{debug, warn};

use crate::collector::{Collector, GpuSnapshot};
use crate::error::{ExporterError, Result};

/// AMD GPU metrics collector powered by AMD SMI (via C++ bridge).
pub struct AmdCollector {
    device_count: u32,
    hostname: String,
}

impl AmdCollector {
    /// Create a new uninitialized AMD collector.
    pub fn new() -> Self {
        Self {
            device_count: 0,
            hostname: whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string()),
        }
    }
}

#[async_trait]
impl Collector for AmdCollector {
    fn name(&self) -> &'static str {
        "amd"
    }

    async fn init(&mut self) -> Result<usize> {
        let ret = is_amd_ffi::amd_smi_init();
        if ret != 0 {
            return Err(ExporterError::AmdInit(ret));
        }

        let count = is_amd_ffi::amd_smi_get_device_count();
        if count == 0 {
            return Err(ExporterError::AmdInit(-1));
        }

        self.device_count = count;
        Ok(count as usize)
    }

    async fn collect(&self) -> Result<Vec<GpuSnapshot>> {
        let mut snapshots = Vec::with_capacity(self.device_count as usize);

        for index in 0..self.device_count {
            let raw = is_amd_ffi::amd_smi_collect_device(index);

            let uuid = raw.uuid.to_string();
            let brand = raw.brand.to_string();

            if uuid.is_empty() || uuid == "INVALID_HANDLE" || uuid == "AMDSMI_ERROR" {
                warn!(index, "AMD device returned invalid UUID, skipping");
                continue;
            }

            let mut snapshot =
                GpuSnapshot::new(index, "amd", self.hostname.clone(), brand, uuid);

            if raw.gpu_utilization_percent >= 0 {
                snapshot.gpu_utilization_percent = Some(raw.gpu_utilization_percent);
            }
            if raw.memory_utilization_percent >= 0 {
                snapshot.memory_utilization_percent = Some(raw.memory_utilization_percent);
            }
            if raw.memory_total_bytes > 0 {
                snapshot.memory_total_bytes = Some(raw.memory_total_bytes);
            }
            if raw.memory_used_bytes > 0 {
                snapshot.memory_used_bytes = Some(raw.memory_used_bytes);
            }
            if raw.power_usage_mw > 0 {
                snapshot.power_usage_mw = Some(raw.power_usage_mw);
            }
            if raw.power_limit_mw > 0 {
                snapshot.power_limit_mw = Some(raw.power_limit_mw);
            }
            if raw.clock_core_mhz > 0 {
                snapshot.clock_core_mhz = Some(raw.clock_core_mhz);
            }
            if raw.clock_memory_mhz > 0 {
                snapshot.clock_memory_mhz = Some(raw.clock_memory_mhz);
            }
            if raw.temperature_celsius > -274 {
                snapshot.temperature_celsius = Some(raw.temperature_celsius);
            }
            if raw.fan_speed_rpm > 0 {
                snapshot.fan_speed = Some(raw.fan_speed_rpm);
            }

            // Compute free memory if we have total and used
            if let (Some(total), Some(used)) = (snapshot.memory_total_bytes, snapshot.memory_used_bytes) {
                snapshot.memory_free_bytes = Some(total.saturating_sub(used));
            }

            debug!(index, vendor = "amd", "Collected GPU metrics");
            snapshots.push(snapshot);
        }

        Ok(snapshots)
    }
}
