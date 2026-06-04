//! Professional htop-like TUI rendering for GPU & system monitoring.
//!
//! Layout (Overview tab):
//! ┌─────────────────────────────────────────────────────────────────┐
//! │ Header: InferSight-top │ hostname │ uptime │ load │ time        │
//! ├────────────────────────────────┬────────────────────────────────┤
//! │ CPU cores (htop-style bars)    │ Memory / Swap / Disk           │
//! ├────────────────────────────────┴────────────────────────────────┤
//! │ GPU cards overview (all GPUs with bars)                          │
//! ├─────────────────────────────────────────────────────────────────┤
//! │ Sparklines: CPU history │ Network I/O                           │
//! ├─────────────────────────────────────────────────────────────────┤
//! │ Footer: keybindings                                              │
//! └─────────────────────────────────────────────────────────────────┘

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph, Sparkline, Tabs},
    Frame,
};

use crate::app::{App, ViewTab};

// ─── Color Palette ──────────────────────────────────────────────────────────

const CYAN: Color = Color::Cyan;
const GREEN: Color = Color::Green;
const YELLOW: Color = Color::Yellow;
const RED: Color = Color::Red;
const MAGENTA: Color = Color::Magenta;
const BLUE: Color = Color::Blue;
const WHITE: Color = Color::White;
const GRAY: Color = Color::Gray;
const DARK_GRAY: Color = Color::DarkGray;
const ORANGE: Color = Color::Rgb(255, 165, 0);

// ─── Helper Functions ───────────────────────────────────────────────────────

/// Format bytes to human-readable.
fn fmt_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1}G", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1}M", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1}K", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}

/// Format bytes with full suffix.
fn fmt_bytes_long(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GiB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MiB", bytes as f64 / 1_048_576.0)
    } else {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    }
}

/// Format network rate.
fn fmt_rate(bytes_per_sec: u64) -> String {
    if bytes_per_sec >= 1_073_741_824 {
        format!("{:.1} GB/s", bytes_per_sec as f64 / 1_073_741_824.0)
    } else if bytes_per_sec >= 1_048_576 {
        format!("{:.1} MB/s", bytes_per_sec as f64 / 1_048_576.0)
    } else if bytes_per_sec >= 1024 {
        format!("{:.1} KB/s", bytes_per_sec as f64 / 1024.0)
    } else {
        format!("{} B/s", bytes_per_sec)
    }
}

/// Format uptime.
fn fmt_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    if days > 0 {
        format!("{days}d {hours:02}:{mins:02}")
    } else {
        format!("{hours:02}:{mins:02}")
    }
}

/// Color based on percentage (htop-style gradient).
fn pct_color(pct: f64) -> Color {
    match pct as u32 {
        0..=25 => GREEN,
        26..=50 => CYAN,
        51..=75 => YELLOW,
        76..=90 => ORANGE,
        _ => RED,
    }
}

/// Temperature color.
fn temp_color(temp: i64) -> Color {
    match temp {
        t if t <= 45 => GREEN,
        t if t <= 65 => YELLOW,
        t if t <= 80 => ORANGE,
        _ => RED,
    }
}

/// Build an htop-style bar string: [||||||||      ]
fn htop_bar(width: usize, percent: f64) -> (String, String) {
    let filled = ((percent / 100.0) * width as f64).round() as usize;
    let filled = filled.min(width);
    let empty = width.saturating_sub(filled);
    let bar_filled: String = "│".repeat(filled);
    let bar_empty: String = " ".repeat(empty);
    (bar_filled, bar_empty)
}

// ─── Main Draw ──────────────────────────────────────────────────────────────

/// Main render entry point.
pub fn draw(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Main vertical layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Header bar
            Constraint::Length(1),  // Tab bar
            Constraint::Min(10),   // Content
            Constraint::Length(2), // Footer
        ])
        .split(size);

    draw_header_bar(frame, app, chunks[0]);
    draw_tab_bar(frame, app, chunks[1]);

    match app.current_tab {
        ViewTab::Overview => draw_overview(frame, app, chunks[2]),
        ViewTab::GpuDetail => draw_gpu_detail(frame, app, chunks[2]),
        ViewTab::Processes => draw_processes(frame, app, chunks[2]),
    }

    draw_footer(frame, app, chunks[3]);

    // Help overlay
    if app.show_help {
        draw_help_overlay(frame, size);
    }
}

// ─── Header Bar ─────────────────────────────────────────────────────────────

fn draw_header_bar(frame: &mut Frame, app: &App, area: Rect) {
    let now = chrono::Local::now().format("%H:%M:%S").to_string();

    let spans = vec![
        Span::styled(" InferSight", Style::default().fg(CYAN).bold()),
        Span::styled("-top ", Style::default().fg(BLUE).bold()),
        Span::styled("│ ", Style::default().fg(DARK_GRAY)),
        Span::styled(&app.system.hostname, Style::default().fg(GREEN).bold()),
        Span::styled(" │ ", Style::default().fg(DARK_GRAY)),
        Span::styled("up ", Style::default().fg(DARK_GRAY)),
        Span::styled(
            fmt_uptime(app.system.uptime_seconds),
            Style::default().fg(WHITE),
        ),
        Span::styled(" │ ", Style::default().fg(DARK_GRAY)),
        Span::styled("load ", Style::default().fg(DARK_GRAY)),
        Span::styled(
            format!(
                "{:.2} {:.2} {:.2}",
                app.system.load_avg_1m, app.system.load_avg_5m, app.system.load_avg_15m
            ),
            Style::default().fg(YELLOW),
        ),
        Span::styled(" │ ", Style::default().fg(DARK_GRAY)),
        Span::styled(
            format!("{} procs", app.system.process_count),
            Style::default().fg(WHITE),
        ),
        Span::styled(" │ ", Style::default().fg(DARK_GRAY)),
        Span::styled(
            format!("{} GPU(s)", app.device_count),
            Style::default().fg(MAGENTA).bold(),
        ),
        Span::styled(" │ ", Style::default().fg(DARK_GRAY)),
        Span::styled(
            format!("Driver {} ", app.driver_version),
            Style::default().fg(DARK_GRAY),
        ),
        Span::styled(" │ ", Style::default().fg(DARK_GRAY)),
        Span::styled(now, Style::default().fg(CYAN)),
        Span::raw(" "),
    ];

    let para = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Color::Rgb(30, 30, 46)));
    frame.render_widget(para, area);
}

// ─── Tab Bar ────────────────────────────────────────────────────────────────

fn draw_tab_bar(frame: &mut Frame, app: &App, area: Rect) {
    let titles = vec![
        Line::from(" [1] Overview "),
        Line::from(" [2] GPU Detail "),
        Line::from(" [3] Processes "),
    ];

    let selected = match app.current_tab {
        ViewTab::Overview => 0,
        ViewTab::GpuDetail => 1,
        ViewTab::Processes => 2,
    };

    let tabs = Tabs::new(titles)
        .select(selected)
        .style(Style::default().fg(GRAY).bg(Color::Rgb(40, 40, 55)))
        .highlight_style(Style::default().fg(CYAN).bold().bg(Color::Rgb(60, 60, 80)))
        .divider(symbols::DOT);

    frame.render_widget(tabs, area);
}

// ─── Overview Tab ───────────────────────────────────────────────────────────

fn draw_overview(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(cpu_section_height(app)), // CPU + Mem section
            Constraint::Min(4),                          // GPU overview
            Constraint::Length(6),                        // Sparklines
        ])
        .split(area);

    draw_system_section(frame, app, chunks[0]);
    draw_gpu_overview(frame, app, chunks[1]);
    draw_history_section(frame, app, chunks[2]);
}

/// Calculate needed height for cpu section.
fn cpu_section_height(app: &App) -> u16 {
    let core_rows = (app.cpu_count as u16 + 1) / 2; // 2 columns
    (core_rows + 2).max(7) // +2 for borders, min 7
}

// ─── System Section (CPU bars + Memory/Swap/Disk) ───────────────────────────

fn draw_system_section(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    draw_cpu_bars(frame, app, chunks[0]);
    draw_memory_section(frame, app, chunks[1]);
}

/// Draw htop-style per-core CPU bars.
fn draw_cpu_bars(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DARK_GRAY))
        .title(Span::styled(
            " CPU ",
            Style::default().fg(GREEN).bold(),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // Split into 2 columns for CPU cores
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    let half = (app.per_core_usage.len() + 1) / 2;
    let bar_width = (cols[0].width as usize).saturating_sub(8); // leave room for label

    for (col_idx, col_area) in cols.iter().enumerate() {
        let start = col_idx * half;
        let end = (start + half).min(app.per_core_usage.len());

        let mut lines: Vec<Line> = Vec::new();
        for i in start..end {
            if lines.len() >= col_area.height as usize {
                break;
            }
            let usage = app.per_core_usage[i] as f64;
            let (filled, empty) = htop_bar(bar_width, usage);
            let color = pct_color(usage);

            let line = Line::from(vec![
                Span::styled(
                    format!("{:>2}", i),
                    Style::default().fg(DARK_GRAY),
                ),
                Span::styled("[", Style::default().fg(DARK_GRAY)),
                Span::styled(filled, Style::default().fg(color)),
                Span::styled(empty, Style::default().fg(DARK_GRAY)),
                Span::styled(
                    format!("{:>5.1}%", usage),
                    Style::default().fg(color),
                ),
                Span::styled("]", Style::default().fg(DARK_GRAY)),
            ]);
            lines.push(line);
        }

        let para = Paragraph::new(lines);
        frame.render_widget(para, *col_area);
    }
}

/// Draw memory, swap, and disk usage.
fn draw_memory_section(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DARK_GRAY))
        .title(Span::styled(" Memory ", Style::default().fg(BLUE).bold()));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let sys = &app.system;
    let bar_width = (inner.width as usize).saturating_sub(12);

    let mut lines: Vec<Line> = Vec::new();

    // RAM bar
    let ram_pct = if sys.memory_total_bytes > 0 {
        (sys.memory_used_bytes as f64 / sys.memory_total_bytes as f64) * 100.0
    } else {
        0.0
    };
    let (filled, empty) = htop_bar(bar_width, ram_pct);
    lines.push(Line::from(vec![
        Span::styled("RAM", Style::default().fg(GREEN).bold()),
        Span::styled("[", Style::default().fg(DARK_GRAY)),
        Span::styled(filled, Style::default().fg(pct_color(ram_pct))),
        Span::styled(empty, Style::default().fg(DARK_GRAY)),
        Span::styled(
            format!(
                "{}/{}",
                fmt_bytes(sys.memory_used_bytes),
                fmt_bytes(sys.memory_total_bytes)
            ),
            Style::default().fg(WHITE),
        ),
        Span::styled("]", Style::default().fg(DARK_GRAY)),
    ]));

    // Swap bar
    let swap_pct = if sys.swap_total_bytes > 0 {
        (sys.swap_used_bytes as f64 / sys.swap_total_bytes as f64) * 100.0
    } else {
        0.0
    };
    let (filled, empty) = htop_bar(bar_width, swap_pct);
    lines.push(Line::from(vec![
        Span::styled("SWP", Style::default().fg(YELLOW).bold()),
        Span::styled("[", Style::default().fg(DARK_GRAY)),
        Span::styled(filled, Style::default().fg(pct_color(swap_pct))),
        Span::styled(empty, Style::default().fg(DARK_GRAY)),
        Span::styled(
            format!(
                "{}/{}",
                fmt_bytes(sys.swap_used_bytes),
                fmt_bytes(sys.swap_total_bytes)
            ),
            Style::default().fg(WHITE),
        ),
        Span::styled("]", Style::default().fg(DARK_GRAY)),
    ]));

    // Separator
    lines.push(Line::from(""));

    // Network I/O
    lines.push(Line::from(vec![
        Span::styled("NET", Style::default().fg(MAGENTA).bold()),
        Span::styled(" ▼ ", Style::default().fg(GREEN)),
        Span::styled(fmt_rate(app.net_rx_rate), Style::default().fg(GREEN)),
        Span::styled(" ▲ ", Style::default().fg(RED)),
        Span::styled(fmt_rate(app.net_tx_rate), Style::default().fg(RED)),
    ]));

    // Disk summary
    if let Some(disk) = sys.disks.first() {
        let used = disk.total_bytes.saturating_sub(disk.available_bytes);
        let disk_pct = if disk.total_bytes > 0 {
            (used as f64 / disk.total_bytes as f64) * 100.0
        } else {
            0.0
        };
        lines.push(Line::from(vec![
            Span::styled("DSK", Style::default().fg(CYAN).bold()),
            Span::styled(
                format!(
                    " {}/{} ({:.0}%) {}",
                    fmt_bytes(used),
                    fmt_bytes(disk.total_bytes),
                    disk_pct,
                    disk.mount_point
                ),
                Style::default().fg(WHITE),
            ),
        ]));
    }

    // Temperatures (if any)
    if let Some(temp) = sys.temperatures.first() {
        lines.push(Line::from(vec![
            Span::styled("TMP", Style::default().fg(RED).bold()),
            Span::styled(
                format!(" {}: {:.0}°C", temp.label, temp.current_celsius),
                Style::default().fg(temp_color(temp.current_celsius as i64)),
            ),
        ]));
    }

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner);
}

// ─── GPU Overview (all GPUs at once) ────────────────────────────────────────

fn draw_gpu_overview(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DARK_GRAY))
        .title(Span::styled(
            " GPUs ",
            Style::default().fg(MAGENTA).bold(),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.gpus.is_empty() {
        let msg = Paragraph::new(Line::from(vec![
            Span::styled("  ⚠ ", Style::default().fg(YELLOW)),
            Span::styled(
                "No GPUs detected. Check drivers.",
                Style::default().fg(GRAY),
            ),
        ]));
        frame.render_widget(msg, inner);
        return;
    }

    // Each GPU gets 3 lines
    let gpu_constraints: Vec<Constraint> = app
        .gpus
        .iter()
        .map(|_| Constraint::Length(3))
        .collect();

    let gpu_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(gpu_constraints)
        .split(inner);

    for (i, gpu) in app.gpus.iter().enumerate() {
        if i >= gpu_areas.len() {
            break;
        }
        draw_gpu_row(frame, app, gpu, i, gpu_areas[i]);
    }
}

/// Draw a single GPU row in overview (3 lines: name, util bar, mem bar).
fn draw_gpu_row(frame: &mut Frame, app: &App, gpu: &is_exporter::collector::GpuSnapshot, idx: usize, area: Rect) {
    if area.height < 3 || area.width < 20 {
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Name + stats
            Constraint::Length(1), // GPU util bar
            Constraint::Length(1), // MEM bar
        ])
        .split(area);

    let temp = gpu.temperature_celsius.unwrap_or(0);
    let power_w = gpu.power_usage_mw.unwrap_or(0) / 1000;
    let power_limit_w = gpu.power_limit_mw.unwrap_or(1) / 1000;
    let fan = gpu.fan_speed;
    let selected_marker = if idx == app.selected_gpu { "▶" } else { " " };

    // Row 1: GPU name + key stats
    let name_line = Line::from(vec![
        Span::styled(
            format!("{selected_marker}GPU {idx}"),
            Style::default()
                .fg(if idx == app.selected_gpu { CYAN } else { WHITE })
                .bold(),
        ),
        Span::styled(" │ ", Style::default().fg(DARK_GRAY)),
        Span::styled(
            truncate_str(&gpu.brand, 28),
            Style::default().fg(WHITE),
        ),
        Span::styled(" │ ", Style::default().fg(DARK_GRAY)),
        Span::styled(
            format!("{temp}°C"),
            Style::default().fg(temp_color(temp)),
        ),
        Span::styled(" │ ", Style::default().fg(DARK_GRAY)),
        Span::styled(
            format!("{power_w}/{power_limit_w}W"),
            Style::default().fg(YELLOW),
        ),
        Span::styled(" │ ", Style::default().fg(DARK_GRAY)),
        match fan {
            Some(f) => Span::styled(format!("Fan:{f}%"), Style::default().fg(CYAN)),
            None => Span::styled("Fan:N/A", Style::default().fg(DARK_GRAY)),
        },
        Span::styled(" │ ", Style::default().fg(DARK_GRAY)),
        Span::styled(
            format!(
                "{}MHz/{}MHz",
                gpu.clock_core_mhz.unwrap_or(0),
                gpu.clock_memory_mhz.unwrap_or(0)
            ),
            Style::default().fg(DARK_GRAY),
        ),
    ]);
    frame.render_widget(Paragraph::new(name_line), rows[0]);

    // Row 2: GPU utilization bar
    let gpu_util = gpu.gpu_utilization_percent.unwrap_or(0).max(0) as f64;
    let bar_width = (rows[1].width as usize).saturating_sub(14);
    let (filled, empty) = htop_bar(bar_width, gpu_util);
    let util_line = Line::from(vec![
        Span::styled(" GPU", Style::default().fg(GREEN)),
        Span::styled("[", Style::default().fg(DARK_GRAY)),
        Span::styled(filled, Style::default().fg(pct_color(gpu_util))),
        Span::styled(empty, Style::default().fg(DARK_GRAY)),
        Span::styled(
            format!("{:>5.1}%", gpu_util),
            Style::default().fg(pct_color(gpu_util)),
        ),
        Span::styled("]", Style::default().fg(DARK_GRAY)),
    ]);
    frame.render_widget(Paragraph::new(util_line), rows[1]);

    // Row 3: Memory bar
    let mem_used = gpu.memory_used_bytes.unwrap_or(0);
    let mem_total = gpu.memory_total_bytes.unwrap_or(1);
    let mem_pct = (mem_used as f64 / mem_total as f64) * 100.0;
    let (filled, empty) = htop_bar(bar_width, mem_pct);
    let mem_line = Line::from(vec![
        Span::styled(" MEM", Style::default().fg(BLUE)),
        Span::styled("[", Style::default().fg(DARK_GRAY)),
        Span::styled(filled, Style::default().fg(pct_color(mem_pct))),
        Span::styled(empty, Style::default().fg(DARK_GRAY)),
        Span::styled(
            format!("{}/{}", fmt_bytes(mem_used), fmt_bytes(mem_total)),
            Style::default().fg(WHITE),
        ),
        Span::styled("]", Style::default().fg(DARK_GRAY)),
    ]);
    frame.render_widget(Paragraph::new(mem_line), rows[2]);
}

// ─── History Sparklines Section ─────────────────────────────────────────────

fn draw_history_section(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    // CPU history sparkline
    {
        let data: Vec<u64> = app.cpu_history.iter().map(|v| *v as u64).collect();
        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title(Span::styled(
                        format!(" CPU {:.1}% ", app.system.cpu_usage_percent),
                        Style::default().fg(GREEN),
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(DARK_GRAY)),
            )
            .data(&data)
            .max(100)
            .style(Style::default().fg(GREEN));
        frame.render_widget(sparkline, chunks[0]);
    }

    // GPU util history (selected GPU)
    if app.selected_gpu < app.gpu_util_history.len() {
        let data: Vec<u64> = app.gpu_util_history[app.selected_gpu]
            .iter()
            .map(|v| *v as u64)
            .collect();
        let gpu_util = app.gpus.get(app.selected_gpu)
            .and_then(|g| g.gpu_utilization_percent)
            .unwrap_or(0);
        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title(Span::styled(
                        format!(" GPU{} {}% ", app.selected_gpu, gpu_util),
                        Style::default().fg(MAGENTA),
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(DARK_GRAY)),
            )
            .data(&data)
            .max(100)
            .style(Style::default().fg(MAGENTA));
        frame.render_widget(sparkline, chunks[1]);
    }

    // Network sparkline
    {
        let data: Vec<u64> = app.net_rx_history.iter().map(|v| *v as u64).collect();
        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title(Span::styled(
                        format!(" Net ▼{} ▲{} ", fmt_rate(app.net_rx_rate), fmt_rate(app.net_tx_rate)),
                        Style::default().fg(CYAN),
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(DARK_GRAY)),
            )
            .data(&data)
            .style(Style::default().fg(CYAN));
        frame.render_widget(sparkline, chunks[2]);
    }
}

// ─── GPU Detail Tab ─────────────────────────────────────────────────────────

fn draw_gpu_detail(frame: &mut Frame, app: &App, area: Rect) {
    if app.gpus.is_empty() {
        let msg = Paragraph::new("No GPUs detected")
            .style(Style::default().fg(RED))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" GPU Detail "),
            );
        frame.render_widget(msg, area);
        return;
    }

    let gpu = &app.gpus[app.selected_gpu];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // GPU selector tabs
            Constraint::Length(5),  // Key metrics
            Constraint::Length(3),  // Gauges
            Constraint::Length(3),  // Gauges
            Constraint::Min(5),    // Sparklines
        ])
        .split(area);

    // GPU selector
    draw_gpu_selector(frame, app, chunks[0]);

    // Key metrics panel
    draw_gpu_metrics(frame, gpu, chunks[1]);

    // GPU utilization gauge
    let gpu_util = gpu.gpu_utilization_percent.unwrap_or(0).max(0) as u16;
    let gpu_gauge = Gauge::default()
        .block(
            Block::default()
                .title(Span::styled(" GPU Utilization ", Style::default().fg(GREEN)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(DARK_GRAY)),
        )
        .gauge_style(
            Style::default()
                .fg(pct_color(gpu_util as f64))
                .add_modifier(Modifier::BOLD),
        )
        .percent(gpu_util.min(100))
        .label(format!("{}%", gpu_util));
    frame.render_widget(gpu_gauge, chunks[2]);

    // Memory gauge
    let mem_used = gpu.memory_used_bytes.unwrap_or(0);
    let mem_total = gpu.memory_total_bytes.unwrap_or(1);
    let mem_pct = ((mem_used as f64 / mem_total as f64) * 100.0) as u16;
    let mem_gauge = Gauge::default()
        .block(
            Block::default()
                .title(Span::styled(" VRAM ", Style::default().fg(BLUE)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(DARK_GRAY)),
        )
        .gauge_style(
            Style::default()
                .fg(pct_color(mem_pct as f64))
                .add_modifier(Modifier::BOLD),
        )
        .percent(mem_pct.min(100))
        .label(format!(
            "{} / {} ({:.0}%)",
            fmt_bytes_long(mem_used),
            fmt_bytes_long(mem_total),
            mem_pct
        ));
    frame.render_widget(mem_gauge, chunks[3]);

    // Detail sparklines (4 charts)
    draw_detail_sparklines(frame, app, chunks[4]);
}

fn draw_gpu_selector(frame: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = app
        .gpus
        .iter()
        .enumerate()
        .map(|(i, g)| {
            let temp = g.temperature_celsius.unwrap_or(0);
            Line::from(format!(" GPU{i} {temp}°C "))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(DARK_GRAY))
                .title(Span::styled(
                    " Select GPU [←/→] ",
                    Style::default().fg(CYAN),
                )),
        )
        .select(app.selected_gpu)
        .style(Style::default().fg(GRAY))
        .highlight_style(Style::default().fg(CYAN).bold())
        .divider(symbols::DOT);

    frame.render_widget(tabs, area);
}

fn draw_gpu_metrics(frame: &mut Frame, gpu: &is_exporter::collector::GpuSnapshot, area: Rect) {
    let temp = gpu.temperature_celsius.unwrap_or(0);
    let power_w = gpu.power_usage_mw.unwrap_or(0) as f64 / 1000.0;
    let power_limit_w = gpu.power_limit_mw.unwrap_or(0) as f64 / 1000.0;
    let core_mhz = gpu.clock_core_mhz.unwrap_or(0);
    let mem_mhz = gpu.clock_memory_mhz.unwrap_or(0);
    let fan = gpu.fan_speed;

    let lines = vec![
        Line::from(vec![
            Span::styled(" Name: ", Style::default().fg(DARK_GRAY)),
            Span::styled(&gpu.brand, Style::default().fg(WHITE).bold()),
            Span::styled("    UUID: ", Style::default().fg(DARK_GRAY)),
            Span::styled(
                truncate_str(&gpu.uuid, 36),
                Style::default().fg(DARK_GRAY),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Temp: ", Style::default().fg(DARK_GRAY)),
            Span::styled(
                format!("{temp}°C"),
                Style::default().fg(temp_color(temp)).bold(),
            ),
            Span::styled("    Power: ", Style::default().fg(DARK_GRAY)),
            Span::styled(
                format!("{power_w:.0}/{power_limit_w:.0} W"),
                Style::default().fg(YELLOW).bold(),
            ),
            Span::styled("    Fan: ", Style::default().fg(DARK_GRAY)),
            match fan {
                Some(f) => Span::styled(format!("{f}%"), Style::default().fg(CYAN).bold()),
                None => Span::styled("N/A", Style::default().fg(DARK_GRAY)),
            },
        ]),
        Line::from(vec![
            Span::styled(" Clocks: ", Style::default().fg(DARK_GRAY)),
            Span::styled(
                format!("{core_mhz} MHz"),
                Style::default().fg(CYAN),
            ),
            Span::styled(" core  ", Style::default().fg(DARK_GRAY)),
            Span::styled(
                format!("{mem_mhz} MHz"),
                Style::default().fg(CYAN),
            ),
            Span::styled(" mem", Style::default().fg(DARK_GRAY)),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DARK_GRAY))
        .title(Span::styled(
            format!(" GPU {} ", gpu.index),
            Style::default().fg(MAGENTA).bold(),
        ));

    let para = Paragraph::new(lines).block(block);
    frame.render_widget(para, area);
}

fn draw_detail_sparklines(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    let idx = app.selected_gpu;

    // GPU Util
    if idx < app.gpu_util_history.len() {
        let data: Vec<u64> = app.gpu_util_history[idx].iter().map(|v| *v as u64).collect();
        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title(Span::styled(" Util% ", Style::default().fg(GREEN)))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(DARK_GRAY)),
            )
            .data(&data)
            .max(100)
            .style(Style::default().fg(GREEN));
        frame.render_widget(sparkline, chunks[0]);
    }

    // VRAM Util
    if idx < app.mem_util_history.len() {
        let data: Vec<u64> = app.mem_util_history[idx].iter().map(|v| *v as u64).collect();
        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title(Span::styled(" VRAM% ", Style::default().fg(BLUE)))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(DARK_GRAY)),
            )
            .data(&data)
            .max(100)
            .style(Style::default().fg(BLUE));
        frame.render_widget(sparkline, chunks[1]);
    }

    // Temperature
    if idx < app.gpu_temp_history.len() {
        let data: Vec<u64> = app.gpu_temp_history[idx].iter().map(|v| *v as u64).collect();
        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title(Span::styled(" Temp°C ", Style::default().fg(RED)))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(DARK_GRAY)),
            )
            .data(&data)
            .max(100)
            .style(Style::default().fg(RED));
        frame.render_widget(sparkline, chunks[2]);
    }

    // Power
    if idx < app.gpu_power_history.len() {
        let data: Vec<u64> = app.gpu_power_history[idx].iter().map(|v| *v as u64).collect();
        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title(Span::styled(" Power% ", Style::default().fg(YELLOW)))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(DARK_GRAY)),
            )
            .data(&data)
            .max(100)
            .style(Style::default().fg(YELLOW));
        frame.render_widget(sparkline, chunks[3]);
    }
}

// ─── Processes Tab ──────────────────────────────────────────────────────────

fn draw_processes(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DARK_GRAY))
        .title(Span::styled(
            format!(" Processes ({}) ", app.processes.len()),
            Style::default().fg(GREEN).bold(),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 2 {
        return;
    }

    // Header line
    let header = Line::from(vec![
        Span::styled(
            format!("{:<7}", "PID"),
            Style::default().fg(CYAN).bold(),
        ),
        Span::styled(
            format!("{:<4}", "GPU"),
            Style::default().fg(MAGENTA).bold(),
        ),
        Span::styled(
            format!("{:<5}", "TYPE"),
            Style::default().fg(YELLOW).bold(),
        ),
        Span::styled(
            format!("{:<7}", "CPU%"),
            Style::default().fg(GREEN).bold(),
        ),
        Span::styled(
            format!("{:<8}", "MEM"),
            Style::default().fg(BLUE).bold(),
        ),
        Span::styled(
            format!("{:<9}", "GPU MEM"),
            Style::default().fg(MAGENTA).bold(),
        ),
        Span::styled("COMMAND", Style::default().fg(WHITE).bold()),
    ]);

    let available_rows = (inner.height as usize).saturating_sub(1); // -1 for header
    let scroll = app.scroll_offset as usize;
    let max_scroll = app.processes.len().saturating_sub(available_rows);
    let effective_scroll = scroll.min(max_scroll);

    let mut lines: Vec<Line> = vec![header];

    for proc in app.processes.iter().skip(effective_scroll).take(available_rows) {
        let gpu_str = match proc.gpu_index {
            Some(idx) => format!("{:<4}", idx),
            None => format!("{:<4}", "—"),
        };
        let gpu_mem_str = if proc.gpu_mem_bytes > 0 {
            fmt_bytes(proc.gpu_mem_bytes)
        } else {
            "—".to_string()
        };

        let type_color = match proc.proc_type.as_str() {
            "C" => GREEN,
            "G" => BLUE,
            "C+G" => MAGENTA,
            _ => GRAY,
        };

        let cpu_color = pct_color(proc.cpu_percent as f64);

        let line = Line::from(vec![
            Span::styled(
                format!("{:<7}", proc.pid),
                Style::default().fg(WHITE),
            ),
            Span::styled(
                gpu_str,
                Style::default().fg(if proc.gpu_index.is_some() { MAGENTA } else { DARK_GRAY }),
            ),
            Span::styled(
                format!("{:<5}", proc.proc_type),
                Style::default().fg(type_color),
            ),
            Span::styled(
                format!("{:<7.1}", proc.cpu_percent),
                Style::default().fg(cpu_color),
            ),
            Span::styled(
                format!("{:<8}", fmt_bytes(proc.mem_bytes)),
                Style::default().fg(BLUE),
            ),
            Span::styled(
                format!("{:<9}", gpu_mem_str),
                Style::default().fg(MAGENTA),
            ),
            Span::styled(
                truncate_str(&proc.name, 30),
                Style::default().fg(if proc.gpu_index.is_some() { WHITE } else { GRAY }),
            ),
        ]);
        lines.push(line);
    }

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner);
}

// ─── Footer ─────────────────────────────────────────────────────────────────

fn draw_footer(frame: &mut Frame, _app: &App, area: Rect) {
    let keys = Line::from(vec![
        Span::styled(" q", Style::default().fg(RED).bold()),
        Span::styled("/Esc ", Style::default().fg(DARK_GRAY)),
        Span::styled("Quit", Style::default().fg(GRAY)),
        Span::styled(" │ ", Style::default().fg(DARK_GRAY)),
        Span::styled("Tab", Style::default().fg(CYAN).bold()),
        Span::styled("/1/2 ", Style::default().fg(DARK_GRAY)),
        Span::styled("Switch View", Style::default().fg(GRAY)),
        Span::styled(" │ ", Style::default().fg(DARK_GRAY)),
        Span::styled("←/→", Style::default().fg(GREEN).bold()),
        Span::styled("/h/l ", Style::default().fg(DARK_GRAY)),
        Span::styled("GPU", Style::default().fg(GRAY)),
        Span::styled(" │ ", Style::default().fg(DARK_GRAY)),
        Span::styled("r", Style::default().fg(YELLOW).bold()),
        Span::styled(" ", Style::default().fg(DARK_GRAY)),
        Span::styled("Refresh", Style::default().fg(GRAY)),
        Span::styled(" │ ", Style::default().fg(DARK_GRAY)),
        Span::styled("?", Style::default().fg(MAGENTA).bold()),
        Span::styled(" ", Style::default().fg(DARK_GRAY)),
        Span::styled("Help", Style::default().fg(GRAY)),
    ]);

    let info = Line::from(vec![
        Span::styled(
            " InferSight v0.1.0 ",
            Style::default().fg(DARK_GRAY),
        ),
        Span::styled("│ ", Style::default().fg(DARK_GRAY)),
        Span::styled(
            "github.com/shaswot16/InferSight",
            Style::default().fg(DARK_GRAY),
        ),
    ]);

    let para = Paragraph::new(vec![keys, info])
        .style(Style::default().bg(Color::Rgb(30, 30, 46)));
    frame.render_widget(para, area);
}

// ─── Help Overlay ───────────────────────────────────────────────────────────

fn draw_help_overlay(frame: &mut Frame, area: Rect) {
    let popup_width = 50u16.min(area.width.saturating_sub(4));
    let popup_height = 16u16.min(area.height.saturating_sub(4));
    let popup_area = centered_rect(popup_width, popup_height, area);

    frame.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Keybindings", Style::default().fg(CYAN).bold()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  q / Esc     ", Style::default().fg(RED).bold()),
            Span::styled("Quit application", Style::default().fg(WHITE)),
        ]),
        Line::from(vec![
            Span::styled("  Tab / 1 / 2 ", Style::default().fg(CYAN).bold()),
            Span::styled("Switch between tabs", Style::default().fg(WHITE)),
        ]),
        Line::from(vec![
            Span::styled("  ← / → / h/l ", Style::default().fg(GREEN).bold()),
            Span::styled("Select GPU", Style::default().fg(WHITE)),
        ]),
        Line::from(vec![
            Span::styled("  r           ", Style::default().fg(YELLOW).bold()),
            Span::styled("Force refresh", Style::default().fg(WHITE)),
        ]),
        Line::from(vec![
            Span::styled("  ?           ", Style::default().fg(MAGENTA).bold()),
            Span::styled("Toggle this help", Style::default().fg(WHITE)),
        ]),
        Line::from(vec![
            Span::styled("  j / k       ", Style::default().fg(BLUE).bold()),
            Span::styled("Scroll (if applicable)", Style::default().fg(WHITE)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Press any key to close",
                Style::default().fg(DARK_GRAY),
            ),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(CYAN))
        .title(Span::styled(
            " Help ",
            Style::default().fg(CYAN).bold(),
        ))
        .style(Style::default().bg(Color::Rgb(30, 30, 46)));

    let para = Paragraph::new(help_text).block(block);
    frame.render_widget(para, popup_area);
}

// ─── Utilities ──────────────────────────────────────────────────────────────

/// Create a centered rectangle.
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

/// Truncate a string to max len with ellipsis.
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len.saturating_sub(1)])
    }
}
