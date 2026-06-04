//! System metrics collection — delegates to is-exporter's SystemCollector.

use is_exporter::collector::system::{SystemCollector as ExporterSystemCollector, SystemSnapshot};

/// System collector wrapping the exporter's implementation.
pub struct SystemCollector {
    inner: ExporterSystemCollector,
}

impl SystemCollector {
    pub fn new() -> Self {
        Self {
            inner: ExporterSystemCollector::new(),
        }
    }

    /// Collect system metrics using the exporter's collector.
    pub fn collect(&self) -> SystemSnapshot {
        self.inner.collect_system_snapshot()
    }
}
