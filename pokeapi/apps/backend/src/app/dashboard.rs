//! Dashboard: buscador de pokémon (con origen caché/API visible) e
//! historial de consultas recientes sacado de Redis.

use leptos::prelude::*;
use leptos_router::components::Redirect;

use super::RecursoSesion;
use super::api::{consultar_pokemon, historial_reciente, limpiar_historial};
use super::modelos::{ConsultaUi, FichaUi, SesionUi, formatear_nombre, mensaje_error};

#[component]
pub fn PaginaDashboard() -> impl IntoView {
    let sesion = expect_context::<RecursoSesion>();
    view! {
        <Suspense fallback=|| view! { <p class="cargando">"Cargando…"</p> }>
            {move || {
                sesion.get().map(|estado| match estado {
                    None => view! { <Redirect path="/login"/> }.into_any(),
                    Some(usuario) => view! { <Dashboard usuario=usuario/> }.into_any(),
                })
            }}
        </Suspense>
    }
}

#[component]
fn Dashboard(usuario: SesionUi) -> impl IntoView {
    let consultar = Action::new(|nombre: &String| consultar_pokemon(nombre.clone()));
    // Cada consulta resuelta (cambia `version`) refresca el historial.
    let historial = Resource::new(
        move || consultar.version().get(),
        |_| async move { historial_reciente().await.unwrap_or_default() },
    );
    let texto = RwSignal::new(String::new());
    let puede_editar = usuario.puede_editar();

    let limpiar = Action::new(|(): &()| limpiar_historial());
    Effect::new(move |_| {
        if let Some(Ok(())) = limpiar.value().get() {
            historial.refetch();
        }
    });

    view! {
        <section class="panel">
            <div class="columna-principal">
                <form
                    class="buscador"
                    on:submit=move |ev| {
                        ev.prevent_default();
                        let nombre = texto.get();
                        if !nombre.trim().is_empty() {
                            consultar.dispatch(nombre);
                        }
                    }
                >
                    <input
                        type="text"
                        prop:value=texto
                        on:input:target=move |ev| texto.set(ev.target().value())
                        placeholder="pikachu, charizard, mewtwo, snorlax…"
                    />
                    <button type="submit" disabled=move || consultar.pending().get()>
                        {move || if consultar.pending().get() { "Buscando…" } else { "Buscar" }}
                    </button>
                </form>

                {move || match consultar.value().get() {
                    None => {
                        view! {
                            <p class="pista">
                                "Busca un pokémon: la primera consulta va a la PokeAPI (🌐) y las siguientes salen del caché en Redis (⚡). Todo queda en /metrics."
                            </p>
                        }
                            .into_any()
                    }
                    Some(Ok(ficha)) => view! { <TarjetaPokemon ficha=ficha/> }.into_any(),
                    Some(Err(error)) => {
                        view! { <p class="aviso-error">{mensaje_error(&error)}</p> }.into_any()
                    }
                }}
            </div>

            <aside class="columna-lateral">
                <div class="encabezado-lateral">
                    <h2>"Consultas recientes"</h2>
                    <div class="acciones">
                        <button
                            class="secundario"
                            on:click=move |_| {
                                historial.refetch();
                            }
                        >
                            "Actualizar"
                        </button>
                        <Show when=move || puede_editar>
                            <button
                                class="peligro"
                                on:click=move |_| {
                                    limpiar.dispatch(());
                                }
                            >
                                "Limpiar"
                            </button>
                        </Show>
                    </div>
                </div>
                <Suspense fallback=|| view! { <p class="cargando">"Cargando…"</p> }>
                    {move || {
                        historial
                            .get()
                            .map(|consultas| view! { <TablaHistorial consultas=consultas/> })
                    }}
                </Suspense>
            </aside>
        </section>
    }
}

#[component]
fn TarjetaPokemon(ficha: FichaUi) -> impl IntoView {
    let desde_cache = ficha.origen == "cache";
    let etiqueta_origen = if desde_cache {
        format!("⚡ caché Redis · {} ms", ficha.duracion_ms)
    } else {
        format!("🌐 pokeapi.co · {} ms", ficha.duracion_ms)
    };

    view! {
        <article class="tarjeta pokemon">
            <div class="cabecera-pokemon">
                <h1>
                    {formatear_nombre(&ficha.nombre)}
                    <span class="numero">{format!(" #{:03}", ficha.numero)}</span>
                </h1>
                <span class=if desde_cache {
                    "chip origen-cache"
                } else {
                    "chip origen-api"
                }>{etiqueta_origen}</span>
            </div>
            <div class="cuerpo-pokemon">
                {ficha
                    .sprite_url
                    .clone()
                    .map(|url| view! { <img class="sprite" src=url alt=ficha.nombre.clone()/> })}
                <div class="datos">
                    <p class="tipos">
                        {ficha
                            .tipos
                            .iter()
                            .map(|tipo| {
                                view! { <span class=format!("chip tipo-{tipo}")>{tipo.clone()}</span> }
                            })
                            .collect_view()}
                    </p>
                    <p class="medidas">
                        {format!(
                            "Altura {:.1} m · Peso {:.1} kg",
                            f64::from(ficha.altura_dm) / 10.0,
                            f64::from(ficha.peso_hg) / 10.0,
                        )}
                    </p>
                    <ul class="estadisticas">
                        {ficha
                            .estadisticas
                            .iter()
                            .map(|est| {
                                let ancho = format!(
                                    "width: {:.0}%",
                                    (f64::from(est.valor).min(200.0) / 2.0),
                                );
                                view! {
                                    <li>
                                        <span class="nombre-stat">{est.nombre.clone()}</span>
                                        <span class="valor-stat">{est.valor}</span>
                                        <div class="pista-barra">
                                            <div class="barra" style=ancho></div>
                                        </div>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                </div>
            </div>
        </article>
    }
}

#[component]
fn TablaHistorial(consultas: Vec<ConsultaUi>) -> impl IntoView {
    if consultas.is_empty() {
        return view! { <p class="pista">"Aún no hay consultas."</p> }.into_any();
    }
    view! {
        <ul class="historial">
            {consultas
                .into_iter()
                .map(|consulta| {
                    let clase_fila = if consulta.exito { "fila" } else { "fila fallida" };
                    let icono = if consulta.origen == "cache" { "⚡" } else { "🌐" };
                    view! {
                        <li class=clase_fila>
                            <span class="hora">{consulta.hora()}</span>
                            <span class="quien">
                                {consulta.usuario.clone()}
                                <span class=format!(
                                    "chip mini rol-{}",
                                    consulta.rol.to_lowercase(),
                                )>{consulta.rol.clone()}</span>
                            </span>
                            <span class="que">{consulta.pokemon.clone()}</span>
                            <span class="origen">{icono}</span>
                        </li>
                    }
                })
                .collect_view()}
        </ul>
    }
    .into_any()
}
