#pragma once

/// @file amd_smi_wrapper.h
/// @brief Clean C++ wrapper for AMD SMI, exposing a structured API to Rust via CXX.
///
/// Design principles:
/// - No global mutable state (metrics map eliminated)
/// - Single function per device returns all metrics at once
/// - RAII-style initialization/shutdown
/// - All errors handled gracefully with fallback values

#include <cstdint>
#include <string>
#include <vector>

#include "rust/cxx.h"
#include "amd_smi/amdsmi.h"

/// Metrics snapshot for a single AMD GPU device.
/// This struct is returned directly to Rust via CXX bridge.
struct AmdDeviceMetrics {
    rust::String uuid;
    rust::String brand;
    int64_t gpu_utilization_percent;
    int64_t memory_utilization_percent;
    uint64_t memory_total_bytes;
    uint64_t memory_used_bytes;
    uint64_t power_usage_mw;
    uint64_t power_limit_mw;
    uint32_t clock_core_mhz;
    uint32_t clock_memory_mhz;
    int64_t temperature_celsius;
    uint32_t fan_speed_rpm;
};

/// Initialize AMD SMI library.
/// @return 0 on success, non-zero error code on failure.
int32_t amd_smi_init();

/// Shutdown AMD SMI library and release resources.
void amd_smi_shutdown();

/// Get the number of AMD GPU devices available.
/// @return Number of GPU devices, or 0 if none/error.
uint32_t amd_smi_get_device_count();

/// Collect all metrics for a single device.
/// @param device_index Zero-based device index.
/// @return Populated AmdDeviceMetrics struct (with fallback values on errors).
AmdDeviceMetrics amd_smi_collect_device(uint32_t device_index);
