//! Global Prometheus registry.

use once_cell::sync::Lazy;
use prometheus::Registry;

/// Global Prometheus registry shared across the application.
pub static REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);
