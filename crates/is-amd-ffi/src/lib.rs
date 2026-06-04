//! AMD SMI FFI bridge — shared library for AMD GPU operations.
//!
//! This crate provides the CXX bridge to the AMD SMI C++ library.
//! It can be used by both the exporter (for metrics) and the control CLI (for power management).
//!
//! # Usage
//! ```rust,ignore
//! use is_amd_ffi::{amd_smi_init, amd_smi_get_device_count, amd_smi_collect_device};
//!
//! let rc = amd_smi_init();
//! if rc == 0 {
//!     let count = amd_smi_get_device_count();
//!     for i in 0..count {
//!         let metrics = amd_smi_collect_device(i);
//!         println!("GPU {}: {} — {}°C", i, metrics.brand, metrics.temperature_celsius);
//!     }
//! }
//! ```

#[cfg(all(feature = "amd", target_arch = "x86_64"))]
#[cxx::bridge]
pub mod bridge {
    /// Raw device metrics returned from the C++ AMD SMI wrapper.
    #[derive(Debug, Clone)]
    struct AmdDeviceMetrics {
        pub uuid: String,
        pub brand: String,
        pub gpu_utilization_percent: i64,
        pub memory_utilization_percent: i64,
        pub memory_total_bytes: u64,
        pub memory_used_bytes: u64,
        pub power_usage_mw: u64,
        pub power_limit_mw: u64,
        pub clock_core_mhz: u32,
        pub clock_memory_mhz: u32,
        pub temperature_celsius: i64,
        pub fan_speed_rpm: u32,
    }

    unsafe extern "C++" {
        include!("amd_smi_wrapper.h");

        /// Initialize AMD SMI library. Returns 0 on success.
        fn amd_smi_init() -> i32;

        /// Shutdown AMD SMI library.
        fn amd_smi_shutdown();

        /// Get the number of AMD GPU devices detected.
        fn amd_smi_get_device_count() -> u32;

        /// Collect all metrics for a single device by index.
        fn amd_smi_collect_device(device_index: u32) -> AmdDeviceMetrics;

        /// Set performance level for a device.
        /// level: "auto", "low", "high", "manual"
        /// Returns true on success.
        fn amd_smi_set_perf_level(device_index: u32, level: &CxxString) -> bool;

        /// Set power limit in milliwatts.
        /// Returns 0 on success, non-zero on failure.
        fn amd_smi_set_power_limit(device_index: u32, power_limit_mw: u64) -> i32;

        /// Get current perf level as integer (0=auto, 1=low, 2=high, 3=manual).
        fn amd_smi_get_perf_level(device_index: u32) -> i32;

        /// Set clock level for a device.
        /// uuid: device UUID (or "Default" for all)
        /// level: -1=auto, 0=low, 1=high
        /// expiry_secs: seconds before auto-reset (0 = no reset)
        /// Returns 0 on success.
        fn amd_smi_control_clk_level(uuid: &CxxString, level: i32, expiry_secs: u64) -> i32;
    }
}

// Re-export functions for easier access
#[cfg(all(feature = "amd", target_arch = "x86_64"))]
pub use bridge::*;

// ─── Stub implementations for non-AMD platforms ─────────────────────────────

#[cfg(not(all(feature = "amd", target_arch = "x86_64")))]
pub mod bridge {
    #[derive(Debug, Clone)]
    pub struct AmdDeviceMetrics {
        pub uuid: String,
        pub brand: String,
        pub gpu_utilization_percent: i64,
        pub memory_utilization_percent: i64,
        pub memory_total_bytes: u64,
        pub memory_used_bytes: u64,
        pub power_usage_mw: u64,
        pub power_limit_mw: u64,
        pub clock_core_mhz: u32,
        pub clock_memory_mhz: u32,
        pub temperature_celsius: i64,
        pub fan_speed_rpm: u32,
    }
}

#[cfg(not(all(feature = "amd", target_arch = "x86_64")))]
pub fn amd_smi_init() -> i32 {
    -1 // Not supported
}

#[cfg(not(all(feature = "amd", target_arch = "x86_64")))]
pub fn amd_smi_shutdown() {}

#[cfg(not(all(feature = "amd", target_arch = "x86_64")))]
pub fn amd_smi_get_device_count() -> u32 {
    0
}

#[cfg(not(all(feature = "amd", target_arch = "x86_64")))]
pub fn amd_smi_collect_device(_device_index: u32) -> bridge::AmdDeviceMetrics {
    bridge::AmdDeviceMetrics {
        uuid: String::new(),
        brand: String::new(),
        gpu_utilization_percent: -1,
        memory_utilization_percent: -1,
        memory_total_bytes: 0,
        memory_used_bytes: 0,
        power_usage_mw: 0,
        power_limit_mw: 0,
        clock_core_mhz: 0,
        clock_memory_mhz: 0,
        temperature_celsius: -999,
        fan_speed_rpm: 0,
    }
}

#[cfg(not(all(feature = "amd", target_arch = "x86_64")))]
pub fn amd_smi_set_perf_level(_device_index: u32, _level: &str) -> bool {
    false
}

#[cfg(not(all(feature = "amd", target_arch = "x86_64")))]
pub fn amd_smi_set_power_limit(_device_index: u32, _power_limit_mw: u64) -> i32 {
    -1
}

#[cfg(not(all(feature = "amd", target_arch = "x86_64")))]
pub fn amd_smi_get_perf_level(_device_index: u32) -> i32 {
    -1
}

#[cfg(not(all(feature = "amd", target_arch = "x86_64")))]
pub fn amd_smi_control_clk_level(_uuid: &str, _level: i32, _expiry_secs: u64) -> i32 {
    -1
}
