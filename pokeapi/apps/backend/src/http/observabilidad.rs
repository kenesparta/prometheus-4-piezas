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
    let redis_ok = pong.is_ok();

    let ping_mongo = estado
        .contenedor
        .mongo
        .run_command(mongodb::bson::doc! { "ping": 1 })
        .await;
    let mongo_ok = ping_mongo.is_ok();

    let (codigo, estado_txt) = if redis_ok && mongo_ok {
        (StatusCode::OK, "ok")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "degradado")
    };
    (
        codigo,
        axum::Json(json!({
            "estado": estado_txt,
            "redis": if redis_ok { "ok" } else { "caído" },
            "mongo": if mongo_ok { "ok" } else { "caído" },
        })),
    )
        .into_response()
}
