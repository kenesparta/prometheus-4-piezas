//! Backend de la demo "PokeAPI × Prometheus en 4 piezas".
//!
//! Crate doble:
//! - Como **librería wasm** (feature `hydrate`) contiene solo la UI Leptos
//!   (`app/`) y se hidrata en el navegador.
//! - Como **binario** (feature `ssr`) aloja todos los adaptadores
//!   (persistencia Redis, cliente PokeAPI, métricas Prometheus, HTTP) y el
//!   wiring de los Bounded Contexts.

pub mod app;

#[cfg(feature = "ssr")]
pub mod clientes;
#[cfg(feature = "ssr")]
pub mod composicion;
#[cfg(feature = "ssr")]
pub mod configuracion;
#[cfg(feature = "ssr")]
pub mod http;
#[cfg(feature = "ssr")]
pub mod mensajeria;
#[cfg(feature = "ssr")]
pub mod metricas;
#[cfg(feature = "ssr")]
pub mod persistencia;
#[cfg(feature = "ssr")]
pub mod seguridad;
#[cfg(feature = "ssr")]
pub mod sesion;

/// Punto de entrada del artefacto wasm: hidrata el HTML que ya llegó del
/// servidor.
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(app::App);
}
