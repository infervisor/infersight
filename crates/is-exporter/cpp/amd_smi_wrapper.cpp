/// @file amd_smi_wrapper.cpp
/// @brief Professional AMD SMI wrapper implementation.
///
/// This replaces the original 1200+ line rocm_wrapper.cc with a clean,
/// structured design that:
/// - Returns all metrics in a single struct per device (no global maps)
/// - Caches processor handles for performance
/// - Uses RAII patterns for initialization
/// - Has proper error handling with fallback values

#include "amd_smi_wrapper.h"

#include <iostream>
#include <mutex>
#include <vector>

// ============================================================================
// Internal state (file-scoped, not exported)
// ============================================================================

namespace {

/// Cached processor handles for all detected GPUs.
struct GpuContext {
    std::vector<amdsmi_processor_handle> handles;
    bool initialized = false;
    std::mutex mutex;
};

static GpuContext g_ctx;

/// Safely get a processor handle for a device index.
/// @return true if handle is valid, false otherwise.
bool get_handle(uint32_t device_index, amdsmi_processor_handle& out_handle) {
    std::lock_guard<std::mutex> lock(g_ctx.mutex);
    if (!g_ctx.initialized || device_index >= g_ctx.handles.size()) {
        return false;
    }
    out_handle = g_ctx.handles[device_index];
    return true;
}

/// Enumerate all GPU processor handles across all sockets.
bool enumerate_gpus() {
    uint32_t socket_count = 0;
    if (amdsmi_get_socket_handles(&socket_count, nullptr) != AMDSMI_STATUS_SUCCESS) {
        std::cerr << "[amd_smi] Failed to get socket count\n";
        return false;
    }
    if (socket_count == 0) {
        std::cerr << "[amd_smi] No sockets found\n";
        return false;
    }

    std::vector<amdsmi_socket_handle> sockets(socket_count);
    if (amdsmi_get_socket_handles(&socket_count, sockets.data()) != AMDSMI_STATUS_SUCCESS) {
        std::cerr << "[amd_smi] Failed to enumerate sockets\n";
        return false;
    }

    g_ctx.handles.clear();

    for (uint32_t i = 0; i < socket_count; ++i) {
        uint32_t device_count = 0;
        if (amdsmi_get_processor_handles(sockets[i], &device_count, nullptr) != AMDSMI_STATUS_SUCCESS) {
            continue;
        }

        std::vector<amdsmi_processor_handle> procs(device_count);
        if (amdsmi_get_processor_handles(sockets[i], &device_count, procs.data()) != AMDSMI_STATUS_SUCCESS) {
            continue;
        }

        for (uint32_t j = 0; j < device_count; ++j) {
            processor_type_t ptype;
            if (amdsmi_get_processor_type(procs[j], &ptype) == AMDSMI_STATUS_SUCCESS &&
                ptype == AMDSMI_PROCESSOR_TYPE_AMD_GPU) {
                g_ctx.handles.push_back(procs[j]);
            }
        }
    }

    return !g_ctx.handles.empty();
}

// ============================================================================
// Metric collection helpers (internal)
// ============================================================================

int64_t collect_gpu_busy(amdsmi_processor_handle h) {
    uint32_t busy = 0;
    if (amdsmi_get_gpu_busy_percent(h, &busy) == AMDSMI_STATUS_SUCCESS) {
        return static_cast<int64_t>(busy);
    }
    return -1;
}

void collect_memory(amdsmi_processor_handle h, uint64_t& total, uint64_t& used) {
    total = 0;
    used = 0;
    uint64_t val = 0;

    if (amdsmi_get_gpu_memory_total(h, AMDSMI_MEM_TYPE_VRAM, &val) == AMDSMI_STATUS_SUCCESS) {
        total = val;
    }
    if (amdsmi_get_gpu_memory_usage(h, AMDSMI_MEM_TYPE_VRAM, &val) == AMDSMI_STATUS_SUCCESS) {
        used = val;
    }
}

int64_t collect_memory_utilization(uint64_t total, uint64_t used) {
    if (total == 0) return -1;
    return static_cast<int64_t>((100.0 * used) / total);
}

void collect_power(amdsmi_processor_handle h, uint64_t& usage_mw, uint64_t& limit_mw) {
    usage_mw = 0;
    limit_mw = 0;

    amdsmi_power_info_t power_info{};
    if (amdsmi_get_power_info(h, &power_info) == AMDSMI_STATUS_SUCCESS) {
        // current_socket_power is in watts from AMD SMI
        usage_mw = static_cast<uint64_t>(power_info.current_socket_power) * 1000;
        limit_mw = static_cast<uint64_t>(power_info.power_limit) * 1000;
    }
}

uint32_t collect_clock(amdsmi_processor_handle h, amdsmi_clk_type_t clk_type) {
    amdsmi_frequencies_t freq_info{};
    if (amdsmi_get_clk_freq(h, clk_type, &freq_info) == AMDSMI_STATUS_SUCCESS) {
        uint32_t level = freq_info.current;
        if (level < freq_info.num_supported && freq_info.num_supported > 0) {
            // Frequency is in Hz, convert to MHz
            return static_cast<uint32_t>(freq_info.frequency[level] / 1000000);
        }
    }
    return 0;
}

int64_t collect_temperature(amdsmi_processor_handle h) {
    // Try junction temperature first (hotspot), then edge
    int64_t temp = 0;
    if (amdsmi_get_temp_metric(h, AMDSMI_TEMPERATURE_TYPE_JUNCTION,
                                AMDSMI_TEMP_CURRENT, &temp) == AMDSMI_STATUS_SUCCESS) {
        return temp / 1000; // millidegrees to degrees
    }
    if (amdsmi_get_temp_metric(h, AMDSMI_TEMPERATURE_TYPE_EDGE,
                                AMDSMI_TEMP_CURRENT, &temp) == AMDSMI_STATUS_SUCCESS) {
        return temp / 1000;
    }
    return -999; // Sentinel: invalid
}

uint32_t collect_fan_rpm(amdsmi_processor_handle h) {
    int64_t rpm = 0;
    if (amdsmi_get_gpu_fan_rpms(h, 0, &rpm) == AMDSMI_STATUS_SUCCESS) {
        return static_cast<uint32_t>(rpm);
    }
    return 0;
}

rust::String collect_uuid(amdsmi_processor_handle h) {
    char uuid_buf[64] = {0};
    unsigned int uuid_len = sizeof(uuid_buf);
    if (amdsmi_get_gpu_device_uuid(h, &uuid_len, uuid_buf) == AMDSMI_STATUS_SUCCESS) {
        return rust::String(uuid_buf);
    }
    return rust::String("UNKNOWN");
}

rust::String collect_brand(amdsmi_processor_handle h) {
    amdsmi_board_info_t board_info{};
    if (amdsmi_get_gpu_board_info(h, &board_info) == AMDSMI_STATUS_SUCCESS) {
        return rust::String(board_info.product_name);
    }
    return rust::String("AMD GPU");
}

} // anonymous namespace

// ============================================================================
// Public API (exported to Rust via CXX)
// ============================================================================

int32_t amd_smi_init() {
    std::lock_guard<std::mutex> lock(g_ctx.mutex);

    amdsmi_status_t ret = amdsmi_init(AMDSMI_INIT_AMD_GPUS);
    if (ret != AMDSMI_STATUS_SUCCESS) {
        std::cerr << "[amd_smi] amdsmi_init failed with code: " << ret << "\n";
        return static_cast<int32_t>(ret);
    }

    if (!enumerate_gpus()) {
        std::cerr << "[amd_smi] No AMD GPUs found during enumeration\n";
        return -1;
    }

    g_ctx.initialized = true;
    std::cout << "[amd_smi] Initialized successfully with "
              << g_ctx.handles.size() << " GPU(s)\n";
    return 0;
}

void amd_smi_shutdown() {
    std::lock_guard<std::mutex> lock(g_ctx.mutex);
    if (g_ctx.initialized) {
        amdsmi_shut_down();
        g_ctx.handles.clear();
        g_ctx.initialized = false;
    }
}

uint32_t amd_smi_get_device_count() {
    std::lock_guard<std::mutex> lock(g_ctx.mutex);
    return static_cast<uint32_t>(g_ctx.handles.size());
}

AmdDeviceMetrics amd_smi_collect_device(uint32_t device_index) {
    AmdDeviceMetrics metrics{};
    metrics.gpu_utilization_percent = -1;
    metrics.memory_utilization_percent = -1;
    metrics.memory_total_bytes = 0;
    metrics.memory_used_bytes = 0;
    metrics.power_usage_mw = 0;
    metrics.power_limit_mw = 0;
    metrics.clock_core_mhz = 0;
    metrics.clock_memory_mhz = 0;
    metrics.temperature_celsius = -999;
    metrics.fan_speed_rpm = 0;

    amdsmi_processor_handle handle;
    if (!get_handle(device_index, handle)) {
        metrics.uuid = rust::String("INVALID_HANDLE");
        metrics.brand = rust::String("Unknown");
        return metrics;
    }

    // Identity
    metrics.uuid = collect_uuid(handle);
    metrics.brand = collect_brand(handle);

    // Utilization
    metrics.gpu_utilization_percent = collect_gpu_busy(handle);

    // Memory
    collect_memory(handle, metrics.memory_total_bytes, metrics.memory_used_bytes);
    metrics.memory_utilization_percent =
        collect_memory_utilization(metrics.memory_total_bytes, metrics.memory_used_bytes);

    // Power
    collect_power(handle, metrics.power_usage_mw, metrics.power_limit_mw);

    // Clocks
    metrics.clock_core_mhz = collect_clock(handle, AMDSMI_CLK_TYPE_SYS);
    metrics.clock_memory_mhz = collect_clock(handle, AMDSMI_CLK_TYPE_DF);

    // Temperature
    metrics.temperature_celsius = collect_temperature(handle);

    // Fan
    metrics.fan_speed_rpm = collect_fan_rpm(handle);

    return metrics;
}
