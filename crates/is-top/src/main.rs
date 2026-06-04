//! gpu-top — Professional terminal GPU & system monitor.
//!
//! Like htop, but for GPUs and system resources. Real-time monitoring of:
//! • Per-core CPU usage with htop-style colored bars
//! • Memory, swap, disk, network I/O
//! • All GPU utilization, VRAM, temperature, power, clocks
//! • History sparklines for CPU, GPU, and network
//!
//! Controls:
//!   q / Esc      — Quit
//!   Tab / 1 / 2  — Switch tabs (Overview / GPU Detail)
//!   ← / → / h/l — Switch between GPUs
//!   r            — Force refresh
//!   ?            — Toggle help

fn main() -> color_eyre::Result<()> {
    is_top::run()
}
