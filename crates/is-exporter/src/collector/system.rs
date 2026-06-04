//! System metrics collector (CPU, memory, disk, network, temperature).

use async_trait::async_trait;
use sysinfo::{Components, Disks, Networks, System};
use tracing::debug;

use crate::collector::{Collector, GpuSnapshot};
use crate::error::Result;

/// Collected system-level metrics snapshot.
#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub hostname: String,
    pub device_name: String,
    pub cpu_usage_percent: f64,
    pub cpu_core_count: Option<usize>,
    pub memory_total_bytes: u64,
    pub memory_used_bytes: u64,
    pub memory_available_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub uptime_seconds: u64,
    pub load_avg_1m: f64,
    pub load_avg_5m: f64,
    pub load_avg_15m: f64,
    pub process_count: usize,
    pub disks: Vec<DiskInfo>,
    pub networks: Vec<NetworkInfo>,
    pub temperatures: Vec<TemperatureInfo>,
}

/// Disk information.
#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub is_removable: bool,
}

/// Network interface information.
#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub interface: String,
    pub mac_address: String,
    pub received_bytes: u64,
    pub transmitted_bytes: u64,
    pub total_received_bytes: u64,
    pub total_transmitted_bytes: u64,
}

/// Hardware temperature sensor info.
#[derive(Debug, Clone)]
pub struct TemperatureInfo {
    pub label: String,
    pub current_celsius: f64,
    pub max_celsius: Option<f64>,
}

/// System metrics collector.
pub struct SystemCollector {
    hostname: String,
    device_name: String,
}

impl SystemCollector {
    /// Create a new system collector.
    pub fn new() -> Self {
        Self {
            hostname: whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string()),
            device_name: whoami::fallible::devicename()
                .or_else(|_| whoami::fallible::hostname())
                .unwrap_or_else(|_| "unknown".to_string()),
        }
    }

    /// Collect a full system snapshot.
    pub fn collect_system_snapshot(&self) -> SystemSnapshot {
        let mut sys = System::new_all();
        sys.refresh_cpu_all();

        let cpu_usage = sys.global_cpu_usage() as f64;
        let cpu_core_count = System::physical_core_count();
        let memory_total = sys.total_memory();
        let memory_used = sys.used_memory();
        let memory_available = sys.available_memory();
        let swap_total = sys.total_swap();
        let swap_used = sys.used_swap();
        let uptime = System::uptime();
        let load = System::load_average();
        let process_count = sys.processes().len();

        // Disks
        let disk_list = Disks::new_with_refreshed_list();
        let disks: Vec<DiskInfo> = disk_list
            .list()
            .iter()
            .map(|d| DiskInfo {
                name: d.name().to_string_lossy().to_string(),
                mount_point: d.mount_point().to_string_lossy().to_string(),
                file_system: d.file_system().to_string_lossy().to_string(),
                total_bytes: d.total_space(),
                available_bytes: d.available_space(),
                is_removable: d.is_removable(),
            })
            .collect();

        // Networks
        let net_list = Networks::new_with_refreshed_list();
        let networks: Vec<NetworkInfo> = net_list
            .iter()
            .map(|(name, data)| NetworkInfo {
                interface: name.clone(),
                mac_address: data.mac_address().to_string(),
                received_bytes: data.received(),
                transmitted_bytes: data.transmitted(),
                total_received_bytes: data.total_received(),
                total_transmitted_bytes: data.total_transmitted(),
            })
            .collect();

        // Temperatures
        let components = Components::new_with_refreshed_list();
        let temperatures: Vec<TemperatureInfo> = components
            .iter()
            .filter_map(|c| {
                let temp = c.temperature()?;
                Some(TemperatureInfo {
                    label: c.label().to_string(),
                    current_celsius: temp as f64,
                    max_celsius: c.max().map(|m| m as f64),
                })
            })
            .collect();

        SystemSnapshot {
            hostname: self.hostname.clone(),
            device_name: self.device_name.clone(),
            cpu_usage_percent: cpu_usage,
            cpu_core_count,
            memory_total_bytes: memory_total,
            memory_used_bytes: memory_used,
            memory_available_bytes: memory_available,
            swap_total_bytes: swap_total,
            swap_used_bytes: swap_used,
            uptime_seconds: uptime,
            load_avg_1m: load.one,
            load_avg_5m: load.five,
            load_avg_15m: load.fifteen,
            process_count,
            disks,
            networks,
            temperatures,
        }
    }
}

#[async_trait]
impl Collector for SystemCollector {
    fn name(&self) -> &'static str {
        "system"
    }

    async fn init(&mut self) -> Result<usize> {
        // System collector always has exactly 1 "device" (the host)
        debug!("System collector initialized");
        Ok(1)
    }

    async fn collect(&self) -> Result<Vec<GpuSnapshot>> {
        // System collector doesn't produce GpuSnapshots — it uses its own path.
        // We return an empty vec here; the system metrics are exported separately.
        Ok(Vec::new())
    }
}
