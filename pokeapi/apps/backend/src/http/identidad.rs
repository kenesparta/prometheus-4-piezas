//! Adaptador HTTP JSON para el BC identidad.
//!
//! La UI usa server functions; estos endpoints existen para la demo por
//! terminal (registrar usuarios y obtener tokens con `curl`). La traducción
//! de errores a códigos HTTP vive aquí: el dominio no conoce HTTP.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use bc_identidad::aplicacion::casos_uso::ErrorCasoUso;
use bc_identidad::aplicacion::dto::{IniciarSesionCmd, RegistrarUsuarioCmd};
use serde::Deserialize;
use serde_json::json;

use super::EstadoServidor;

pub fn router() -> Router<EstadoServidor> {
    Router::new()
        .route("/api/registro", post(registrar))
        .route("/api/login", post(iniciar_sesion))
}

#[derive(Debug, Deserialize)]
struct Credenciales {
    nombre: String,
    password: String,
}

async fn registrar(
    State(estado): State<EstadoServidor>,
    Json(cuerpo): Json<Credenciales>,
) -> Response {
    let cmd = RegistrarUsuarioCmd { nombre: cuerpo.nombre, password: cuerpo.password };
    match estado.contenedor.identidad.registrar.ejecutar(cmd).await {
        Ok(vista) => (StatusCode::CREATED, Json(json!(vista))).into_response(),
        Err(error) => mapear_error(error),
    }
}

async fn iniciar_sesion(
    State(estado): State<EstadoServidor>,
    Json(cuerpo): Json<Credenciales>,
) -> Response {
    let cmd = IniciarSesionCmd { nombre: cuerpo.nombre, password: cuerpo.password };
    match estado.contenedor.identidad.iniciar_sesion.ejecutar(cmd).await {
        Ok(vista) => (StatusCode::OK, Json(json!(vista))).into_response(),
        Err(error) => mapear_error(error),
    }
}

fn mapear_error(error: ErrorCasoUso) -> Response {
    let (estado, mensaje) = match &error {
        ErrorCasoUso::Dominio(e) => (StatusCode::BAD_REQUEST, e.to_string()),
        ErrorCasoUso::NombreTomado => (StatusCode::CONFLICT, error.to_string()),
        ErrorCasoUso::CredencialesInvalidas => (StatusCode::UNAUTHORIZED, error.to_string()),
        ErrorCasoUso::NoAutorizado => (StatusCode::FORBIDDEN, error.to_string()),
        ErrorCasoUso::NoEncontrado => (StatusCode::NOT_FOUND, error.to_string()),
        ErrorCasoUso::Identidad(e) => (StatusCode::CONFLICT, e.to_string()),
        ErrorCasoUso::Repositorio(e) => {
            tracing::error!(error = %e, "error de repositorio en identidad");
            (StatusCode::INTERNAL_SERVER_ERROR, "error interno".to_string())
        }
        ErrorCasoUso::Hasher(e) => {
            tracing::error!(error = %e, "error del hasher");
            (StatusCode::INTERNAL_SERVER_ERROR, "error interno".to_string())
        }
    };
    (estado, Json(json!({ "error": mensaje }))).into_response()
}
