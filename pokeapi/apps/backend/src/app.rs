//! UI Leptos de la demo: shell HTML, rutas y barra superior.

pub mod admin;
pub mod api;
pub mod dashboard;
pub mod login;
pub mod modelos;
pub mod registro;

use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::StaticSegment;
use leptos_router::components::{A, Redirect, Route, Router, Routes};

use self::api::{cerrar_sesion, obtener_sesion};
use self::modelos::SesionUi;

/// Recurso compartido con la sesión actual (se provee por contexto en `App`).
pub type RecursoSesion = Resource<Option<SesionUi>>;

pub fn shell(opciones: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="es">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=opciones.clone()/>
                <HydrationScripts options=opciones/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let sesion: RecursoSesion =
        Resource::new(|| (), |()| async move { obtener_sesion().await.ok().flatten() });
    provide_context(sesion);

    view! {
        <Stylesheet id="leptos" href="/pkg/pokeapi.css"/>
        <Title text="PokeAPI · Prometheus en 4 piezas"/>
        <Router>
            <BarraSuperior/>
            <main class="contenido">
                <Routes fallback=|| view! { <p class="aviso-error">"Página no encontrada."</p> }>
                    <Route path=StaticSegment("") view=Inicio/>
                    <Route path=StaticSegment("login") view=login::PaginaLogin/>
                    <Route path=StaticSegment("registro") view=registro::PaginaRegistro/>
                    <Route path=StaticSegment("dashboard") view=dashboard::PaginaDashboard/>
                    <Route path=StaticSegment("admin") view=admin::PaginaAdmin/>
                </Routes>
            </main>
        </Router>
    }
}

/// Redirige la raíz según haya o no sesión.
#[component]
fn Inicio() -> impl IntoView {
    let sesion = expect_context::<RecursoSesion>();
    view! {
        <Suspense fallback=|| view! { <p class="cargando">"Cargando…"</p> }>
            {move || {
                sesion.get().map(|estado| match estado {
                    Some(_) => view! { <Redirect path="/dashboard"/> }.into_any(),
                    None => view! { <Redirect path="/login"/> }.into_any(),
                })
            }}
        </Suspense>
    }
}

#[component]
fn BarraSuperior() -> impl IntoView {
    let sesion = expect_context::<RecursoSesion>();
    let salir = Action::new(|(): &()| cerrar_sesion());
    let navegar = leptos_router::hooks::use_navigate();

    Effect::new(move |_| {
        if let Some(Ok(())) = salir.value().get() {
            sesion.refetch();
            navegar("/login", Default::default());
        }
    });

    view! {
        <header class="barra">
            <A href="/" attr:class="marca">
                <span class="pokebola" aria-hidden="true"></span>
                "PokeAPI" <small>"× Prometheus"</small>
            </A>
            <nav>
                <Suspense fallback=|| ()>
                    {move || {
                        sesion.get().map(|estado| match estado {
                            Some(usuario) => {
                                let es_admin = usuario.es_admin();
                                let clase_chip = format!("chip rol-{}", usuario.rol.to_lowercase());
                                view! {
                                    <A href="/dashboard">"Dashboard"</A>
                                    <Show when=move || es_admin>
                                        <A href="/admin">"Admin"</A>
                                    </Show>
                                    <span class="usuario">{usuario.nombre.clone()}</span>
                                    <span class=clase_chip>{usuario.rol.clone()}</span>
                                    <button
                                        class="secundario"
                                        on:click=move |_| {
                                            salir.dispatch(());
                                        }
                                    >
                                        "Salir"
                                    </button>
                                }
                                    .into_any()
                            }
                            None => view! {
                                <A href="/login">"Entrar"</A>
                                <A href="/registro">"Registro"</A>
                            }
                                .into_any(),
                        })
                    }}
                </Suspense>
            </nav>
        </header>
    }
}
