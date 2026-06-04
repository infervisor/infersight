//! GPU data collection — delegates to is-exporter's collectors.

use is_exporter::collector::{Collector, GpuSnapshot};
use is_exporter::collector::nvidia::NvidiaCollector;
use is_nvidia::GpuProcessInfo;

/// GPU collector wrapping the exporter's NvidiaCollector.
pub struct GpuCollector {
    nvidia: NvidiaCollector,
    pub device_count: u32,
    pub driver_version: String,
    pub cuda_version: String,
    initialized: bool,
}

impl GpuCollector {
    /// Initialize GPU collection by delegating to the exporter's collector.
    pub fn new() -> Self {
        let mut nvidia = NvidiaCollector::new();

        // Use tokio runtime to call async init
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");

        let (initialized, device_count) = match rt.block_on(nvidia.init()) {
            Ok(count) => (true, count as u32),
            Err(_) => (false, 0),
        };

        // Get driver/CUDA info from the exporter's public methods
        let (driver_version, cuda_version) = if initialized {
            (
                nvidia.driver_version().unwrap_or_else(|| "N/A".into()),
                nvidia.cuda_version().unwrap_or_else(|| "N/A".into()),
            )
        } else {
            ("N/A".into(), "N/A".into())
        };

        Self {
            nvidia,
            device_count,
            driver_version,
            cuda_version,
            initialized,
        }
    }

    /// Collect current GPU metrics using the exporter's collector.
    pub fn collect(&self) -> Vec<GpuSnapshot> {
        if !self.initialized {
            return Vec::new();
        }

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");

        rt.block_on(self.nvidia.collect()).unwrap_or_default()
    }

    /// Collect GPU processes from NVML.
    pub fn collect_processes(&self) -> Vec<GpuProcessInfo> {
        if !self.initialized {
            return Vec::new();
        }
        self.nvidia.collect_gpu_processes()
    }
}
