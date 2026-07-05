//! Entry point del backend.
//!
//! Único punto del workspace donde se conoce el runtime, el servidor HTTP y
//! Redis. Todo lo que se monta vive en `composicion.rs`; aquí solo se arma el
//! router de Axum + Leptos y se arranca.

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use axum::Router;
    use backend::app::{App, shell};
    use backend::http::EstadoServidor;
    use backend::{composicion, configuracion, http, metricas};
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = configuracion::Configuracion::desde_entorno()?;
    let contenedor = composicion::componer(&config).await?;

    // Configuración de Leptos: en desarrollo la inyecta cargo-leptos por
    // entorno; en el contenedor la fijan las variables LEPTOS_*.
    let conf_leptos = get_configuration(None)?;
    let opciones_leptos = conf_leptos.leptos_options;
    let direccion = opciones_leptos.site_addr;
    let rutas = generate_route_list(App);

    let estado = EstadoServidor {
        opciones_leptos,
        contenedor,
    };

    let aplicacion = Router::new()
        // Server functions de Leptos (con el contenedor en el contexto).
        .route("/api/{*fn_name}", axum::routing::any(http::manejador_server_fns))
        // API JSON "curleable" + /metrics + /salud.
        .merge(http::router_api())
        // Páginas Leptos (SSR + hidratación).
        .leptos_routes_with_context(
            &estado,
            rutas,
            {
                let contenedor = estado.contenedor.clone();
                move || leptos::context::provide_context(contenedor.clone())
            },
            {
                let opciones = estado.opciones_leptos.clone();
                move || shell(opciones.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler::<EstadoServidor, _>(shell))
        // La capa de observabilidad envuelve todo: resuelve la sesión y mide
        // cada petición (pieza 1: esta app es un exporter).
        .layer(axum::middleware::from_fn_with_state(estado.clone(), metricas::capa_http))
        .with_state(estado);

    let listener = tokio::net::TcpListener::bind(&direccion).await?;
    tracing::info!(%direccion, "pokeapi escuchando");
    axum::serve(listener, aplicacion.into_make_service()).await?;
    Ok(())
}

/// El binario solo tiene sentido con la feature `ssr` (lo construye
/// cargo-leptos); esta rama existe para que `cargo check --workspace` no
/// falle con las features por defecto.
#[cfg(not(feature = "ssr"))]
fn main() {}
