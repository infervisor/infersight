# GPU Exporter

Professional Prometheus metrics exporter for **AMD**, **NVIDIA**, **Google Cloud TPU**, and **System** monitoring.

## Features

- **NVIDIA GPU metrics** via NVML (nvml-wrapper)
- **AMD GPU metrics** via AMD SMI (C++ CXX bridge)
- **Google Cloud TPU metrics** via TPU runtime endpoint + GCE metadata API
- **System metrics** via sysinfo (CPU, memory, disk, network, temperature)
- **Prometheus-native** — serves metrics at `/metrics` endpoint
- **Feature flags** — compile only what you need
- **Graceful shutdown** — SIGINT/SIGTERM handling
- **Structured logging** — via `tracing` with configurable levels

## Architecture

```
is-exporter/
├── src/
│   ├── main.rs              # CLI entry point, server setup
│   ├── lib.rs               # Library exports
│   ├── config.rs            # CLI configuration (clap)
│   ├── error.rs             # Error types (thiserror)
│   ├── collector/
│   │   ├── mod.rs           # Collector trait + GpuSnapshot type
│   │   ├── manager.rs       # CollectorManager orchestrator
│   │   ├── nvidia.rs        # NVIDIA collector (NVML)
│   │   ├── amd.rs           # AMD collector (CXX bridge)
│   │   ├── tpu.rs           # Google Cloud TPU collector
│   │   └── system.rs        # System metrics collector
│   ├── metrics/
│   │   ├── mod.rs           # Metrics module
│   │   ├── registry.rs      # Global Prometheus registry
│   │   ├── gpu_metrics.rs   # GPU metric definitions
│   │   └── system_metrics.rs # System metric definitions
│   └── exporter/
│       ├── mod.rs           # Exporter module
│       └── prometheus.rs    # HTTP handlers (Axum)
├── cpp/
│   ├── amd_smi_wrapper.h    # C++ header for AMD SMI
│   └── amd_smi_wrapper.cpp  # C++ implementation (~250 lines vs 1200+)
├── amd_smi/                  # AMD SMI vendor headers
├── Cargo.toml
└── build.rs                  # Conditional C++ compilation
```

## Building

### Default (NVIDIA + System):
```bash
cargo build --release
```

### With AMD support:
```bash
cargo build --release --features amd
```

### With TPU support:
```bash
cargo build --release --features tpu
```

### All features:
```bash
cargo build --release --features "nvidia,amd,system,tpu"
```

## Usage

```bash
# Enable all collectors on default port (9835)
is-exporter --all

# NVIDIA only on custom port
is-exporter --nvidia --port 9100

# AMD + System with debug logging
is-exporter --amd --system --log-level debug

# Google Cloud TPU (on a TPU VM)
is-exporter --tpu --system

# Show help
is-exporter --help
```

### CLI Options

| Flag | Description | Default |
|------|-------------|---------|
| `--port` / `-p` | Prometheus HTTP port | `9835` |
| `--interval` / `-i` | Collection interval (seconds) | `5` |
| `--nvidia` | Enable NVIDIA collector | off |
| `--amd` | Enable AMD collector | off |
| `--tpu` | Enable Google Cloud TPU collector | off |
| `--system` | Enable system collector | off |
| `--all` | Enable all collectors | off |
| `--bind` | HTTP bind address | `0.0.0.0` |
| `--log-level` | Log verbosity | `info` |

## Metrics

### GPU Metrics (per device)

| Metric | Description | Labels |
|--------|-------------|--------|
| `gpu_utilization_percent` | Core utilization % | gpu_index, vendor, hostname, brand, uuid |
| `gpu_memory_utilization_percent` | Memory utilization % | ... |
| `gpu_memory_total_bytes` | Total VRAM | ... |
| `gpu_memory_used_bytes` | Used VRAM | ... |
| `gpu_memory_free_bytes` | Free VRAM | ... |
| `gpu_power_usage_watts` | Power draw (W) | ... |
| `gpu_power_limit_watts` | Power limit (W) | ... |
| `gpu_clock_core_mhz` | Core clock (MHz) | ... |
| `gpu_clock_memory_mhz` | Memory clock (MHz) | ... |
| `gpu_temperature_celsius` | Temperature (°C) | ... |
| `gpu_fan_speed` | Fan speed | ... |

### System Metrics

| Metric | Description |
|--------|-------------|
| `system_cpu_usage_percent` | Global CPU usage |
| `system_cpu_core_count` | Physical cores |
| `system_memory_total_bytes` | Total RAM |
| `system_memory_used_bytes` | Used RAM |
| `system_memory_available_bytes` | Available RAM |
| `system_uptime_seconds` | Uptime |
| `system_load_average` | Load avg (1m/5m/15m) |
| `system_disk_*` | Disk metrics |
| `system_network_*` | Network metrics |
| `system_component_temperature_*` | Hardware temps |

## Design Decisions

1. **Trait-based collectors** — `Collector` trait for uniform interface
2. **No global mutable state** — metrics updated via explicit function calls
3. **Feature flags** — don't compile what you don't need
4. **Clean C++ wrapper** — ~250 lines vs 1200+, no global `unordered_map`
5. **Proper error handling** — `thiserror` types, graceful fallbacks
6. **Structured logging** — `tracing` with env-filter support
