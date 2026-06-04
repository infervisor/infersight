//! Export subcommand — starts the Prometheus metrics exporter.
//! Mirrors the logic in is-exporter's main.rs.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use axum::{Router, routing::get};
use tracing::{info, warn};

use is_exporter::collector;
use is_exporter::collector::manager::CollectorManager;
use is_exporter::collector::Collector;
use is_exporter::exporter::prometheus::metrics_handler;
use is_exporter::metrics::gpu_metrics::update_gpu_metrics;

pub fn run(nvidia: bool, amd: bool, system: bool, all: bool, port: u16, bind: String) -> Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    rt.block_on(async move {
        let enable_nvidia = nvidia || all;
        let enable_amd = amd || all;
        let enable_system = system || all;

        // If nothing explicitly enabled, default to nvidia + system
        let (enable_nvidia, enable_system) = if !nvidia && !amd && !system && !all {
            (true, true)
        } else {
            (enable_nvidia, enable_system)
        };

        let mut manager = CollectorManager::new();

        if enable_nvidia {
            info!("Registering NVIDIA collector...");
            let nvidia_c = collector::nvidia::NvidiaCollector::new();
            if let Err(e) = manager.register(Box::new(nvidia_c)).await {
                warn!(error = %e, "NVIDIA collector failed to initialize");
            }
        }

        if enable_amd {
            info!("AMD collector requested");
            // Will use is-amd-ffi when --features amd is enabled
        }

        let interval = Duration::from_secs(5);
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
            warn!("No GPU collectors active");
        }

        // System metrics
        if enable_system {
            use is_exporter::collector::system::SystemCollector;
            use is_exporter::metrics::system_metrics::update_system_metrics;

            let mut sys = SystemCollector::new();
            match Collector::init(&mut sys).await {
                Ok(_) => {
                    tokio::spawn(async move {
                        info!("System metrics collection loop started");
                        loop {
                            let snapshot = sys.collect_system_snapshot();
                            update_system_metrics(&snapshot);
                            tokio::time::sleep(interval).await;
                        }
                    });
                }
                Err(e) => warn!("System collector failed: {e}"),
            }
        }

        // Health handler
        async fn health_handler() -> &'static str {
            "OK"
        }

        let app = Router::new()
            .route("/metrics", get(metrics_handler))
            .route("/health", get(health_handler))
            .route("/healthz", get(health_handler));

        let addr: SocketAddr = format!("{bind}:{port}")
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid address: {e}"))?;

        info!(%addr, "HTTP server starting");
        info!("Prometheus metrics at http://{}/metrics", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                tokio::signal::ctrl_c().await.ok();
                info!("Shutting down...");
            })
            .await
            .map_err(|e| anyhow::anyhow!("Server error: {e}"))?;

        Ok(())
    })
}
