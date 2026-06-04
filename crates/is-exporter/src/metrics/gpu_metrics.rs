//! GPU-specific Prometheus metrics (shared across AMD and NVIDIA).

use once_cell::sync::Lazy;
use prometheus::{IntGaugeVec, Opts};

use super::registry::REGISTRY;

/// Labels applied to all GPU metrics.
const GPU_LABELS: &[&str] = &["gpu_index", "vendor", "hostname", "brand", "uuid"];

/// Helper to create and register an IntGaugeVec with standard GPU labels.
fn gpu_gauge(name: &str, help: &str) -> IntGaugeVec {
    let gauge = IntGaugeVec::new(Opts::new(name, help), GPU_LABELS)
        .unwrap_or_else(|e| panic!("Failed to create metric '{name}': {e}"));
    REGISTRY
        .register(Box::new(gauge.clone()))
        .unwrap_or_else(|e| panic!("Failed to register metric '{name}': {e}"));
    gauge
}

/// All GPU-related Prometheus metrics.
pub struct GpuMetrics {
    pub gpu_utilization_percent: IntGaugeVec,
    pub memory_utilization_percent: IntGaugeVec,
    pub memory_total_bytes: IntGaugeVec,
    pub memory_used_bytes: IntGaugeVec,
    pub memory_free_bytes: IntGaugeVec,
    pub power_usage_watts: IntGaugeVec,
    pub power_limit_watts: IntGaugeVec,
    pub clock_core_mhz: IntGaugeVec,
    pub clock_memory_mhz: IntGaugeVec,
    pub temperature_celsius: IntGaugeVec,
    pub fan_speed: IntGaugeVec,
}

impl GpuMetrics {
    fn new() -> Self {
        Self {
            gpu_utilization_percent: gpu_gauge(
                "gpu_utilization_percent",
                "GPU core utilization percentage (0-100)",
            ),
            memory_utilization_percent: gpu_gauge(
                "gpu_memory_utilization_percent",
                "GPU memory controller utilization percentage (0-100)",
            ),
            memory_total_bytes: gpu_gauge(
                "gpu_memory_total_bytes",
                "Total GPU memory in bytes",
            ),
            memory_used_bytes: gpu_gauge(
                "gpu_memory_used_bytes",
                "Used GPU memory in bytes",
            ),
            memory_free_bytes: gpu_gauge(
                "gpu_memory_free_bytes",
                "Free GPU memory in bytes",
            ),
            power_usage_watts: gpu_gauge(
                "gpu_power_usage_watts",
                "Current GPU power draw in watts",
            ),
            power_limit_watts: gpu_gauge(
                "gpu_power_limit_watts",
                "GPU power limit in watts",
            ),
            clock_core_mhz: gpu_gauge(
                "gpu_clock_core_mhz",
                "GPU core/graphics clock speed in MHz",
            ),
            clock_memory_mhz: gpu_gauge(
                "gpu_clock_memory_mhz",
                "GPU memory clock speed in MHz",
            ),
            temperature_celsius: gpu_gauge(
                "gpu_temperature_celsius",
                "GPU temperature in degrees Celsius",
            ),
            fan_speed: gpu_gauge(
                "gpu_fan_speed",
                "GPU fan speed (percentage for NVIDIA, RPM for AMD)",
            ),
        }
    }
}

/// Global GPU metrics instance.
pub static GPU_METRICS: Lazy<GpuMetrics> = Lazy::new(GpuMetrics::new);

/// Update all GPU Prometheus gauges from a collection of snapshots.
pub fn update_gpu_metrics(snapshots: &[crate::collector::GpuSnapshot]) {
    for snap in snapshots {
        let labels = &[
            snap.index.to_string(),
            snap.vendor.to_string(),
            snap.hostname.clone(),
            snap.brand.clone(),
            snap.uuid.clone(),
        ];
        let label_refs: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();

        if let Some(v) = snap.gpu_utilization_percent {
            GPU_METRICS
                .gpu_utilization_percent
                .with_label_values(&label_refs)
                .set(v);
        }
        if let Some(v) = snap.memory_utilization_percent {
            GPU_METRICS
                .memory_utilization_percent
                .with_label_values(&label_refs)
                .set(v);
        }
        if let Some(v) = snap.memory_total_bytes {
            GPU_METRICS
                .memory_total_bytes
                .with_label_values(&label_refs)
                .set(v as i64);
        }
        if let Some(v) = snap.memory_used_bytes {
            GPU_METRICS
                .memory_used_bytes
                .with_label_values(&label_refs)
                .set(v as i64);
        }
        if let Some(v) = snap.memory_free_bytes {
            GPU_METRICS
                .memory_free_bytes
                .with_label_values(&label_refs)
                .set(v as i64);
        }
        if let Some(v) = snap.power_usage_mw {
            // Convert milliwatts to watts
            GPU_METRICS
                .power_usage_watts
                .with_label_values(&label_refs)
                .set((v / 1000) as i64);
        }
        if let Some(v) = snap.power_limit_mw {
            GPU_METRICS
                .power_limit_watts
                .with_label_values(&label_refs)
                .set((v / 1000) as i64);
        }
        if let Some(v) = snap.clock_core_mhz {
            GPU_METRICS
                .clock_core_mhz
                .with_label_values(&label_refs)
                .set(v as i64);
        }
        if let Some(v) = snap.clock_memory_mhz {
            GPU_METRICS
                .clock_memory_mhz
                .with_label_values(&label_refs)
                .set(v as i64);
        }
        if let Some(v) = snap.temperature_celsius {
            GPU_METRICS
                .temperature_celsius
                .with_label_values(&label_refs)
                .set(v);
        }
        if let Some(v) = snap.fan_speed {
            GPU_METRICS
                .fan_speed
                .with_label_values(&label_refs)
                .set(v as i64);
        }
    }
}
