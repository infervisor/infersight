//! GPU Exporter — Professional Prometheus metrics exporter for GPU and system monitoring.
//!
//! Usage:
//!   is-exporter --all                    # Enable all collectors
//!   is-exporter --nvidia --system        # NVIDIA + system only
//!   is-exporter --amd --port 9100        # AMD on custom port

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::{Router, routing::get};
use clap::Parser;
use tracing::{error, info, warn};

// Use the library crate so all Lazy statics are shared
use is_exporter::collector;
use is_exporter::collector::manager::CollectorManager;
use is_exporter::config::Config;
use is_exporter::exporter::prometheus::{health_handler, metrics_handler};
use is_exporter::metrics::gpu_metrics::update_gpu_metrics;

#[cfg(feature = "system")]
use is_exporter::collector::system::SystemCollector;
#[cfg(feature = "system")]
use is_exporter::metrics::system_metrics::update_system_metrics;

#[cfg(feature = "tpu")]
use is_exporter::collector::tpu::TpuCollector;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse();

    // Initialize tracing/logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| config.log_level.clone().into()),
        )
        .with_target(true)
        .with_thread_ids(true)
        .init();

    info!(
        version = env!("CARGO_PKG_VERSION"),
        "Starting GPU Exporter"
    );

    // Validate configuration
    if config.no_collectors_enabled() {
        warn!("No collectors enabled! Use --nvidia, --amd, --system, or --all");
        warn!("Run with --help for usage information");
        info!("Defaulting to --all");
    }

    // Build collector manager
    let mut manager = CollectorManager::new();

    // Register NVIDIA collector
    #[cfg(feature = "nvidia")]
    if config.nvidia_enabled() || config.no_collectors_enabled() {
        info!("Registering NVIDIA collector...");
        let nvidia = collector::nvidia::NvidiaCollector::new();
        if let Err(e) = manager.register(Box::new(nvidia)).await {
            warn!(error = %e, "NVIDIA collector failed to initialize (continuing without it)");
        }
    }

    // Register AMD collector
    #[cfg(feature = "amd")]
    if config.amd_enabled() || config.no_collectors_enabled() {
        info!("Registering AMD collector...");
        let amd = collector::amd::AmdCollector::new();
        if let Err(e) = manager.register(Box::new(amd)).await {
            warn!(error = %e, "AMD collector failed to initialize (continuing without it)");
        }
    }

    // Register TPU collector
    #[cfg(feature = "tpu")]
    if config.tpu_enabled() || config.no_collectors_enabled() {
        info!("Registering Google Cloud TPU collector...");
        let tpu = TpuCollector::new();
        if let Err(e) = manager.register(Box::new(tpu)).await {
            warn!(error = %e, "TPU collector failed to initialize (continuing without it)");
        }
    }

    // Register System collector
    #[cfg(feature = "system")]
    let system_collector = if config.system_enabled() || config.no_collectors_enabled() {
        info!("Registering System collector...");
        let mut sys = SystemCollector::new();
        match collector::Collector::init(&mut sys).await {
            Ok(_) => {
                info!("System collector initialized");
                Some(sys)
            }
            Err(e) => {
                warn!(error = %e, "System collector failed to initialize");
                None
            }
        }
    } else {
        None
    };

    let interval = Duration::from_secs(config.interval);
    let manager = Arc::new(manager);

    // Spawn GPU metrics collection loop
    if manager.collector_count() > 0 {
        let manager_clone = Arc::clone(&manager);
        tokio::spawn(async move {
            info!("GPU metrics collection loop started");
            loop {
                let snapshots = manager_clone.collect_all().await;
                update_gpu_metrics(&snapshots);
                tokio::time::sleep(interval).await;
            }
        });
    } else {
        warn!("No GPU collectors active — only system metrics will be available");
    }

    // Spawn system metrics collection loop
    #[cfg(feature = "system")]
    if let Some(sys_collector) = system_collector {
        let sys_interval = interval;
        tokio::spawn(async move {
            info!("System metrics collection loop started");
            loop {
                let snapshot = sys_collector.collect_system_snapshot();
                update_system_metrics(&snapshot);
                tokio::time::sleep(sys_interval).await;
            }
        });
    }

    // Build HTTP server
    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_handler))
        .route("/healthz", get(health_handler));

    let addr: SocketAddr = format!("{}:{}", config.bind, config.port)
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid bind address: {e}"))?;

    info!(%addr, "HTTP server starting");
    info!("Prometheus metrics available at http://{}/metrics", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| {
            error!(error = %e, "HTTP server error");
            anyhow::anyhow!("Server error: {e}")
        })?;

    info!("Shutting down gracefully...");

    // Cleanup AMD SMI if initialized
    #[cfg(feature = "amd")]
    is_amd_ffi::amd_smi_shutdown();

    Ok(())
}

/// Wait for SIGINT or SIGTERM for graceful shutdown.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Received Ctrl+C"),
        _ = terminate => info!("Received SIGTERM"),
    }
}
