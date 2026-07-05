//! Endpoints de observabilidad: `/metrics` (lo que rasca Prometheus) y
//! `/salud` (liveness/readiness para Kubernetes).

use axum::Router;
use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde_json::json;

use super::EstadoServidor;

pub fn router() -> Router<EstadoServidor> {
    Router::new()
        .route("/metrics", get(metricas))
        .route("/salud", get(salud))
}

/// Pieza 1 en vivo: métricas en texto plano sobre HTTP.
async fn metricas(State(estado): State<EstadoServidor>) -> Response {
    match estado.contenedor.metricas.texto() {
        Ok(cuerpo) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")],
            cuerpo,
        )
            .into_response(),
        Err(error) => {
            tracing::error!(%error, "no se pudieron codificar las métricas");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn salud(State(estado): State<EstadoServidor>) -> Response {
    let mut con = estado.contenedor.redis.clone();
    let pong: redis::RedisResult<String> = redis::cmd("PING").query_async(&mut con).await;

    match pong {
        Ok(_) => (StatusCode::OK, axum::Json(json!({ "estado": "ok", "redis": "ok" })))
            .into_response(),
        Err(error) => (
            StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(json!({ "estado": "degradado", "redis": error.to_string() })),
        )
            .into_response(),
    }
}
