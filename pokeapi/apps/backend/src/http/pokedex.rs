//! Adaptador HTTP JSON para el BC pokedex.
//!
//! `GET /api/pokemon/{nombre}` admite tráfico anónimo a propósito: durante la
//! charla el público puede tirar `curl` en bucle y ver crecer las series con
//! `rol="anonimo"` junto a las de los usuarios logueados.

use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use bc_pokedex::aplicacion::casos_uso::ErrorCasoUso;
use bc_pokedex::aplicacion::dto::ConsultarPokemonCmd;
use serde::Deserialize;
use serde_json::json;

use super::EstadoServidor;
use crate::sesion::SesionActual;

pub fn router() -> Router<EstadoServidor> {
    Router::new()
        .route("/api/pokemon/{nombre}", get(consultar))
        .route("/api/historial", get(historial))
}

async fn consultar(
    State(estado): State<EstadoServidor>,
    Extension(sesion): Extension<SesionActual>,
    Path(nombre): Path<String>,
) -> Response {
    let cmd = ConsultarPokemonCmd {
        nombre,
        usuario: sesion.usuario(),
        rol: sesion.etiqueta_rol(),
    };
    match estado.contenedor.pokedex.consultar.ejecutar(cmd).await {
        Ok(vista) => (StatusCode::OK, Json(json!(vista))).into_response(),
        Err(error) => mapear_error(error),
    }
}

#[derive(Debug, Deserialize)]
struct ParametrosHistorial {
    /// Cuántas consultas recientes devolver (máx. 100).
    n: Option<usize>,
}

async fn historial(
    State(estado): State<EstadoServidor>,
    Query(parametros): Query<ParametrosHistorial>,
) -> Response {
    let limite = parametros.n.unwrap_or(20).min(100);
    match estado.contenedor.pokedex.historial.ejecutar(limite).await {
        Ok(consultas) => (StatusCode::OK, Json(json!(consultas))).into_response(),
        Err(error) => mapear_error(error),
    }
}

fn mapear_error(error: ErrorCasoUso) -> Response {
    let (estado, mensaje) = match &error {
        ErrorCasoUso::Dominio(e) => (StatusCode::BAD_REQUEST, e.to_string()),
        ErrorCasoUso::NoEncontrado => (StatusCode::NOT_FOUND, error.to_string()),
        ErrorCasoUso::Fuente(_) => (StatusCode::BAD_GATEWAY, error.to_string()),
        ErrorCasoUso::Repositorio(e) => {
            tracing::error!(error = %e, "error de repositorio en pokedex");
            (StatusCode::INTERNAL_SERVER_ERROR, "error interno".to_string())
        }
    };
    (estado, Json(json!({ "error": mensaje }))).into_response()
}
