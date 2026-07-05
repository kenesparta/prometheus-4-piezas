//! Adaptadores HTTP (un archivo por Bounded Context + observabilidad).
//!
//! Además de las páginas Leptos, la app expone una API JSON pensada para
//! generar tráfico desde la terminal durante la charla:
//!
//! ```bash
//! curl -s localhost:3000/api/pokemon/pikachu | jq .origen
//! curl -s localhost:3000/api/historial | jq length
//! curl -s localhost:3000/metrics | grep pokeapi_
//! ```

pub mod identidad;
pub mod observabilidad;
pub mod pokedex;

use axum::Router;
use axum::extract::{FromRef, Request, State};
use axum::response::IntoResponse;
use leptos::prelude::LeptosOptions;

use crate::composicion::Contenedor;

/// Estado del router: opciones de Leptos + contenedor de dependencias.
#[derive(Clone, FromRef)]
pub struct EstadoServidor {
    pub opciones_leptos: LeptosOptions,
    pub contenedor: Contenedor,
}

/// Rutas JSON + observabilidad.
pub fn router_api() -> Router<EstadoServidor> {
    Router::new()
        .merge(identidad::router())
        .merge(pokedex::router())
        .merge(observabilidad::router())
}

/// Atiende las server functions de Leptos dejando el `Contenedor` disponible
/// en el contexto reactivo (las funciones lo recuperan con `use_context`).
pub async fn manejador_server_fns(
    State(estado): State<EstadoServidor>,
    peticion: Request,
) -> impl IntoResponse {
    let contenedor = estado.contenedor.clone();
    leptos_axum::handle_server_fns_with_context(
        move || leptos::context::provide_context(contenedor.clone()),
        peticion,
    )
    .await
}
