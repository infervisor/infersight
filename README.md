<p align="center">
  <h1 align="center">InferSight</h1>
  <p align="center">
    <strong>Production-grade GPU observability & control toolkit for heterogeneous compute infrastructure</strong>
  </p>
  <p align="center">
    <a href="https://github.com/infervisor/infersight"><img src="https://img.shields.io/badge/rust-1.75%2B-orange.svg" alt="Rust 1.75+"></a>
    <a href="https://github.com/infervisor/infersight"><img src="https://img.shields.io/badge/platform-linux-lightgrey.svg" alt="Platform: Linux"></a>
    <a href="https://github.com/infervisor/infersight"><img src="https://img.shields.io/badge/version-0.1.0-green.svg" alt="Version: 0.1.0"></a>
  </p>
  <p align="center">
    <a href="#why-infersight">Why InferSight</a> •
    <a href="#features">Features</a> •
    <a href="#quick-start">Quick Start</a> •
    <a href="#architecture">Architecture</a> •
    <a href="#components">Components</a> •
    <a href="#building">Building</a>
  </p>
</p>

---

## Why InferSight?

Modern AI/ML workloads rely heavily on GPU clusters — yet GPU observability remains a significant operational gap. Existing tools are fragmented, vendor-locked, and lack integration with standard monitoring stacks:

| Problem | InferSight Solution |
|---------|-------------------|
| `nvidia-smi` is NVIDIA-only and hard to parse | Unified multi-vendor interface (NVIDIA + AMD + TPU) |
| No native Prometheus metrics for GPUs | First-class `/metrics` endpoint with labeled GPU metrics |
| Separate tools for monitoring vs. control | Single workspace: monitor, export, and control from one toolkit |
| No real-time terminal dashboards for GPUs | `gpu-top` — interactive TUI with sparkline history |
| GPU clock/power management is arcane | `gpu-ctl` — validated, human-friendly clock & power control |
| Python-based tools have high overhead | Zero-overhead Rust — single static binaries, no runtime deps |

InferSight is designed for **SREs, ML engineers, and platform teams** who need production-grade GPU telemetry that integrates seamlessly with Prometheus/Grafana, Kubernetes, and modern infrastructure tooling.

---

## Features

- **Multi-vendor GPU support** — NVIDIA (NVML), AMD (ROCm SMI), Google Cloud TPU
- **Prometheus-native metrics** — drop-in `/metrics` endpoint for Grafana dashboards
- **Interactive TUI monitor** — real-time gauges, sparkline history, color-coded thresholds
- **GPU power & clock control** — validated writes with hardware constraint checking
- **Feature-flag compilation** — build only what your hardware needs
- **Zero-config auto-detection** — discovers available GPUs at runtime
- **Graceful degradation** — missing vendors don't crash the process
- **Structured logging** — `tracing` with env-filter for debugging
- **Workspace monorepo** — shared dependencies, single lockfile, DRY codebase
 
---

## Quick Start

```bash
# Clone the repository
git clone https://github.com/infervisor/infersight.git
cd infersight

# Build all binaries
cargo build --release

# Start the Prometheus exporter (auto-detects GPUs)
./target/release/gpu-exporter --all

# In another terminal — launch the interactive monitor
./target/release/gpu-top

# Control GPU clocks (requires root)
sudo ./target/release/gpu-ctl nvidia set-clocks 0 --mem 2619 --graphics 1785
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                                InferSight Workspace                              │
│                                                                                 │
│  ┌────────────────────┐   ┌────────────────────┐   ┌────────────────────────┐  │
│  │    is-nvidia        │   │    is-amd-ffi      │   │      is-ctl            │  │
│  │  (shared NVML lib)  │   │  (C++ FFI bridge)  │   │  (control library)     │  │
│  └────────┬───────────┘   └────────┬───────────┘   └────────┬───────────────┘  │
│           │                         │                         │                  │
│  ┌────────┴─────────────────────────┴─────────────────────────┴──────────────┐  │
│  │                            is-exporter                                     │  │
│  │  • Collector trait (NVIDIA, AMD, TPU, System)                              │  │
│  │  • Prometheus metrics registry                                             │  │
│  │  • Axum HTTP server (/metrics, /health, /healthz)                          │  │
│  └────────┬──────────────────────────────────────────────────────────────────┘  │
│           │                                                                     │
│  ┌────────┴───────────┐                    ┌─────────────────────────────────┐  │
│  │      is-top         │                    │          is-cli                 │  │
│  │  (TUI dashboard)    │                    │  (unified `infersight` binary)  │  │
│  │  Reuses exporter    │                    │  export | top | ctl             │  │
│  │  collectors         │                    │                                 │  │
│  └─────────────────────┘                    └─────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## Components

### `gpu-exporter` — Prometheus Metrics Exporter

Exposes GPU and system metrics as Prometheus-compatible time series.

```bash
gpu-exporter --all                     # All collectors, default port 9835
gpu-exporter --nvidia --port 9100      # NVIDIA only, custom port
gpu-exporter --tpu --system            # TPU + system (GCE TPU VMs)
```

**Endpoints:**

| Path | Description |
|------|-------------|
| `/metrics` | Prometheus scrape target |
| `/health` | Health check (HTTP 200) |
| `/healthz` | Kubernetes liveness probe |

**Metrics exposed:** GPU utilization, memory usage/total/free, power draw/limit, temperature, fan speed, clock speeds, per-device labels (UUID, model, hostname).

---

### `gpu-top` — Interactive Terminal Monitor

Real-time GPU dashboard in your terminal — like `htop` but for GPUs.

```bash
gpu-top
```

**Controls:**

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `←` / `h` | Previous GPU |
| `→` / `l` | Next GPU |
| `r` | Force refresh |

**Features:** Color-coded utilization gauges (green → yellow → orange → red), 60-second sparkline history, multi-GPU navigation, system status bar (hostname, CPU, RAM, uptime), driver & CUDA version display.

---

### `gpu-ctl` — GPU Power & Clock Control

Professional GPU management CLI with hardware constraint validation.

```bash
# List all GPUs
gpu-ctl nvidia list
gpu-ctl amd list

# Detailed device info
gpu-ctl nvidia info 0

# Set application clocks (validated against supported ranges)
gpu-ctl nvidia set-clocks 0 --mem 2619 --graphics 1785

# Set power limit
gpu-ctl nvidia set-power-limit 0 --watts 300

# Set performance level
gpu-ctl nvidia set-perf --all --level high
gpu-ctl amd set-perf 0 --level auto

# Reset to defaults
gpu-ctl nvidia reset --all

# Show supported clock combinations
gpu-ctl nvidia supported-clocks 0

# JSON output for scripting
gpu-ctl nvidia list --format json
```

> **Note:** Write operations (set-clocks, set-power-limit, set-perf, reset) require **root/sudo**.

---

### `infersight` — Unified Binary

Single binary combining all functionality:

```bash
infersight export --all          # Start Prometheus exporter
infersight top                   # Launch TUI monitor
infersight ctl nvidia-list       # GPU control commands
```

---

## Building

### Prerequisites

- **Rust 1.75+** (install via [rustup](https://rustup.rs))
- **Linux** with GPU hardware
- **NVIDIA drivers** for NVIDIA GPU support
- **ROCm** for AMD GPU support (optional)
- **GCE TPU VM** for TPU support (optional)

### Build Commands

```bash
# Build all crates (default: NVIDIA + System)
cargo build --release

# Build with specific features
cargo build -p is-exporter --release --features "nvidia,amd,system,tpu"

# Build individual components
cargo build -p is-exporter --release
cargo build -p is-top --release
cargo build -p is-ctl --release

# Run tests
cargo test --workspace

# Check compilation for all features
cargo check -p is-exporter --all-features
```

### Nix

A `flake.nix` is provided for reproducible builds:

```bash
nix build          # Build the unified infersight binary
nix develop        # Enter development shell with Rust toolchain
```

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `nvidia` | ✅ | NVIDIA GPU support via NVML |
| `amd` | ❌ | AMD GPU support via ROCm SMI (C++ FFI) |
| `system` | ✅ | System metrics (CPU, RAM, disk, network) |
| `tpu` | ❌ | Google Cloud TPU support |

---

## Project Structure

```
infersight/
├── Cargo.toml                          # Workspace root
├── flake.nix                           # Nix flake for reproducible builds
├── dashboards/                         # Grafana dashboard JSON
│   └── gpu-system-overview.json
├── crates/
│   ├── is-nvidia/                      # Shared NVIDIA library (NVML)
│   │   └── src/
│   │       ├── lib.rs                  # Public types + stub fallbacks
│   │       └── nvml_impl.rs            # Real NVML implementation
│   │
│   ├── is-amd-ffi/                     # AMD SMI C++ bridge
│   │   ├── cpp/                        # C++ wrapper sources
│   │   ├── amd_smi/                    # Vendor headers
│   │   └── src/lib.rs                  # Rust FFI bindings
│   │
│   ├── is-exporter/                    # Prometheus metrics exporter
│   │   └── src/
│   │       ├── collector/              # Collector trait + vendor impls
│   │       ├── metrics/                # Prometheus metric definitions
│   │       └── exporter/               # HTTP server (Axum)
│   │
│   ├── is-ctl/                         # GPU control library & CLI
│   │   └── src/
│   │       ├── lib.rs                  # Shared types (OutputFormat, PerfLevel, format_bytes)
│   │       ├── nvidia.rs               # NVIDIA control operations
│   │       └── amd.rs                  # AMD control operations
│   │
│   ├── is-top/                         # Terminal GPU monitor (TUI)
│   │   └── src/
│   │       ├── lib.rs                  # App event loop
│   │       ├── ui.rs                   # Ratatui rendering
│   │       └── gpu.rs / system.rs      # Data collection
│   │
│   └── is-cli/                         # Unified binary
│       └── src/
│           ├── main.rs                 # CLI entry (export | top | ctl)
│           ├── cmd_export.rs           # Export subcommand
│           └── cmd_ctl/                # Ctl subcommand (delegates to is-ctl)
│
└── README.md
```

---

## Design Principles

1. **DRY codebase** — Shared libraries (`is-nvidia`, `is-ctl`) eliminate code duplication across binaries
2. **Trait-based architecture** — `Collector` trait provides a uniform vendor-agnostic interface
3. **Feature flags** — Compile only what your target hardware requires
4. **Zero-config defaults** — Auto-detects hardware, gracefully skips unavailable vendors
5. **Professional error handling** — `thiserror` + `anyhow`, no panics in production paths
6. **Structured observability** — `tracing` with env-filter for runtime debugging
7. **Minimal footprint** — Single static binaries, no Python/Node runtime dependencies
8. **Workspace monorepo** — Shared dependencies, single `Cargo.lock`, atomic versioning

---

## Deployment

### Standalone Binary

```bash
cargo build --release
cp target/release/gpu-exporter /usr/local/bin/
cp target/release/gpu-top /usr/local/bin/
cp target/release/gpu-ctl /usr/local/bin/
```

### Kubernetes / Prometheus

```yaml
# prometheus.yml scrape config
scrape_configs:
  - job_name: 'gpu-metrics'
    static_configs:
      - targets: ['gpu-node:9835']
```

### Grafana Dashboard

A pre-built Grafana dashboard is included at [`dashboards/gpu-system-overview.json`](dashboards/gpu-system-overview.json). Import it into your Grafana instance for immediate GPU & system visualization.

### Systemd Service

```ini
[Unit]
Description=InferSight GPU Metrics Exporter
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/gpu-exporter --all
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

