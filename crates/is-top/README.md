# InferSight-top (is-top)

**A professional terminal GPU & system monitor — like `htop`, but for GPUs.**

```
 InferSight-top │ myhost │ up 3d 14:22 │ load 1.23 0.98 0.76 │ 142 procs │ 2 GPU(s) │ 18:45:23
 [1] Overview · [2] GPU Detail
┌ CPU ─────────────────────────────────────┐┌ Memory ───────────────────────────────┐
│ 0[│││││││      23.4%] 8[│││         8.1%]││RAM[│││││││││││││        48.2G/128.0G] │
│ 1[││││││       19.2%] 9[││││       12.3%]││SWP[│                     0.2G/16.0G]  │
│ 2[│││││││││    32.1%]10[│││││││    21.5%]││                                       │
│ 3[││││         14.7%]11[│││││      18.9%]││NET ▼ 12.4 MB/s ▲ 3.2 MB/s            │
│ 4[│││││││││││  45.2%]12[│││        10.2%]││DSK 423.1G/931.5G (45%) /              │
│ 5[││           5.3%] 13[││││││     20.4%]││TMP coretemp: 62°C                     │
│ 6[│││││        16.8%]14[│││││      17.6%]│└───────────────────────────────────────┘
│ 7[││││││       22.0%]15[││          6.9%]│
└──────────────────────────────────────────┘
┌ GPUs ────────────────────────────────────────────────────────────────────────────────┐
│▶GPU 0 │ NVIDIA RTX 4090             │ 72°C │ 285/450W │ Fan:65% │ 2100MHz/10501MHz │
│ GPU[│││││││││││││││││││││││││     62.0%]                                            │
│ MEM[││││││││││││││        18.4G/24.0G]                                              │
│                                                                                      │
│ GPU 1 │ NVIDIA RTX 3090             │ 55°C │ 180/350W │ Fan:40% │ 1800MHz/9501MHz  │
│ GPU[│││││││││              28.0%]                                                    │
│ MEM[││││││                  8.2G/24.0G]                                             │
└──────────────────────────────────────────────────────────────────────────────────────┘
┌ CPU 23.4% ──────────┐┌ GPU0 62% ───────────┐┌ Net ▼12.4 MB/s ▲3.2 MB/s ──────────┐
│ ▁▃▅▇█▇▅▃▁▂▄▆█▇▅▃▁▁ ││ ▁▃▅▇█▇▅▃▁▂▄▆█▇▅▃▁▁ ││ ▂▃▅▆▇▇▆▅▃▂▁▂▃▄▅▆▇▇▆▅▃▂▁          │
└──────────────────────┘└──────────────────────┘└────────────────────────────────────┘
 q/Esc Quit │ Tab/1/2 Switch View │ ←/→/h/l GPU │ r Refresh │ ? Help
 InferSight v0.1.0 │ github.com/infervisor/infersight
```

## What is InferSight-top?

`is-top` (InferSight-top) is a real-time terminal dashboard for monitoring both **GPU** and **system** performance — designed for ML engineers, CUDA developers, DevOps, and anyone working with NVIDIA GPUs. Think of it as `htop` or `btop` but specifically built for GPU workloads with full system context.

It gives you an instant, at-a-glance view of everything happening on your GPUs **and** your system without leaving the terminal.

## Features

### 🖥️ System Monitoring (htop-style)
- **Per-core CPU bars** — colored htop-style bars for every CPU core
- **RAM & Swap** — usage bars with totals
- **Network I/O** — real-time download/upload rates
- **Disk usage** — space used/total with mount point
- **System temperatures** — hardware sensor readings
- **Load average** — 1m, 5m, 15m
- **Process count** — total running processes
- **Uptime** — system uptime

### 🎮 GPU Monitoring
- **All GPUs at once** — overview shows every GPU with utilization bars
- **Temperature** — color-coded (green → yellow → orange → red)
- **Power** — current draw vs. power limit
- **Fan speed** — percentage (if available)
- **Clock speeds** — core and memory frequencies
- **VRAM** — used / total with percentage bar

### 📊 History & Visualization
- **CPU history sparkline** — 120-second rolling graph
- **GPU utilization sparkline** — per-GPU rolling history
- **Network traffic sparkline** — download rate over time
- **GPU temperature history** — track thermal trends
- **GPU power history** — detect power spikes

### 🎛️ Two View Tabs
1. **Overview** — system + all GPUs + sparklines (the htop view)
2. **GPU Detail** — deep-dive into a single GPU with gauges and 4 sparklines

### 🎨 Professional UI
- Catppuccin-inspired dark color scheme
- Color-coded thresholds (green → cyan → yellow → orange → red)
- htop-style `[│││││││    ]` bar rendering
- Tab navigation between views
- Help overlay (`?`)
- Vim-style keybindings (`h/l/j/k`)

## Why InferSight-top?

| Problem | nvidia-smi | nvtop | InferSight-top |
|---------|-----------|-------|---------------|
| Real-time updates | Manual refresh | ✅ | ✅ Auto 1s |
| Visual clarity | Dense text | Good | htop-style bars + sparklines |
| Per-core CPU | ❌ | ❌ | ✅ Full htop-style |
| Memory/Swap/Disk | ❌ | ❌ | ✅ All system info |
| Network I/O | ❌ | ❌ | ✅ Real-time rates |
| History | ❌ | ✅ | ✅ 120s + multiple metrics |
| Multi-GPU | ✅ (text) | ✅ | ✅ Overview + detail |
| Interactive | ❌ | ✅ | ✅ Tabs + vim keys |
| Single binary | ❌ | ❌ | ✅ Rust static |

## Installation

### Build from source (requires Rust):
```bash
cargo build --release -p is-top
# Binary at: ./target/release/is-top
```

### Run directly:
```bash
cargo run --release -p is-top
```

### Install system-wide:
```bash
cargo install --path crates/is-top
```

## Requirements

- **Linux** with NVIDIA GPU(s)
- **NVIDIA driver** installed (any recent version)
- **Terminal** with Unicode support (virtually all modern terminals)
- No CUDA toolkit needed — only the driver

## Usage

```bash
# Just run it
is-top
```

### Keyboard Controls

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `Tab` / `1` / `2` | Switch between Overview and GPU Detail tabs |
| `←` / `h` | Previous GPU |
| `→` / `l` | Next GPU |
| `↑` / `k` | Scroll up |
| `↓` / `j` | Scroll down |
| `r` | Force refresh |
| `?` | Toggle help overlay |

## What You See

### Tab 1: Overview

| Section | Content |
|---------|---------|
| **Header** | Hostname, uptime, load avg, process count, GPU count, driver version, clock |
| **CPU Panel** | Per-core htop-style colored bars in 2 columns |
| **Memory Panel** | RAM bar, Swap bar, Network I/O, Disk usage, Temperatures |
| **GPU Overview** | All GPUs with name, temp, power, fan, clocks + utilization/memory bars |
| **Sparklines** | CPU history, GPU utilization history, Network RX history |
| **Footer** | Keybinding reference + version |

### Tab 2: GPU Detail

| Section | Content |
|---------|---------|
| **GPU Selector** | Tab bar showing all GPUs with temperatures |
| **Metrics** | Name, UUID, temp, power, fan, clocks |
| **GPU Gauge** | Full-width utilization gauge with percentage |
| **VRAM Gauge** | Full-width memory gauge with used/total |
| **Sparklines** | GPU util%, VRAM%, Temperature°C, Power% (4 side-by-side) |

## Architecture

```
is-top/
├── src/
│   ├── main.rs    — Entry point
│   ├── lib.rs     — Terminal setup, event loop, keybindings
│   ├── app.rs     — Application state, tick logic, history tracking
│   ├── gpu.rs     — GPU data collection (via is-exporter)
│   ├── system.rs  — System metrics (via is-exporter)
│   └── ui.rs      — Professional TUI rendering (ratatui)
├── Cargo.toml
└── README.md
```

### Dependencies

| Crate | Purpose |
|-------|---------|
| `ratatui` | Terminal UI framework (widgets, layout, rendering) |
| `crossterm` | Cross-platform terminal manipulation (raw mode, events) |
| `is-exporter` | GPU + System data collection |
| `is-nvidia` | NVIDIA NVML bindings |
| `sysinfo` | Per-core CPU usage |
| `chrono` | Clock display |
| `color-eyre` | Error handling |

## How It Works

1. **Initialization** — connects to NVML, detects GPUs, queries driver/CUDA version, initializes sysinfo
2. **Event loop** — renders at 1 FPS, polls keyboard events between frames
3. **Data collection** — each tick queries GPU metrics (NVML) + system metrics (sysinfo) + network rates
4. **History tracking** — maintains rolling 120-sample buffers for sparklines (CPU, GPU util, GPU mem, GPU temp, GPU power, network)
5. **Rendering** — ratatui builds a widget tree and diffs against the previous frame for efficient terminal updates
