//! Prometheus metrics HTTP handler using Axum.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use prometheus::{Encoder, TextEncoder};

use crate::metrics::registry::REGISTRY;

/// HTTP handler that serves Prometheus metrics in text exposition format.
pub async fn metrics_handler() -> Response {
    let metric_families = REGISTRY.gather();
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();

    match encoder.encode(&metric_families, &mut buffer) {
        Ok(()) => {
            let body = String::from_utf8(buffer).unwrap_or_default();
            (
                StatusCode::OK,
                [("Content-Type", "text/plain; version=0.0.4; charset=utf-8")],
                body,
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to encode metrics: {e}"),
        )
            .into_response(),
    }
}

/// Health check endpoint.
pub async fn health_handler() -> Response {
    (StatusCode::OK, "OK").into_response()
}
