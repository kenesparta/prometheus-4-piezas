//! Panel de administración (solo ADMIN): lista de usuarios y cambio de rol.

use leptos::prelude::*;
use leptos_router::components::Redirect;

use super::RecursoSesion;
use super::api::{cambiar_rol, listar_usuarios};
use super::modelos::{ROLES, mensaje_error};

#[component]
pub fn PaginaAdmin() -> impl IntoView {
    let sesion = expect_context::<RecursoSesion>();
    view! {
        <Suspense fallback=|| view! { <p class="cargando">"Cargando…"</p> }>
            {move || {
                sesion.get().map(|estado| match estado {
                    None => view! { <Redirect path="/login"/> }.into_any(),
                    Some(usuario) if !usuario.es_admin() => {
                        view! { <Redirect path="/dashboard"/> }.into_any()
                    }
                    Some(_) => view! { <PanelAdmin/> }.into_any(),
                })
            }}
        </Suspense>
    }
}

#[component]
fn PanelAdmin() -> impl IntoView {
    let cambiar = Action::new(|entrada: &(String, String)| {
        let (nombre, rol) = entrada.clone();
        cambiar_rol(nombre, rol)
    });
    // Cada cambio aplicado (cambia `version`) recarga la lista.
    let usuarios = Resource::new(
        move || cambiar.version().get(),
        |_| async move { listar_usuarios().await },
    );

    let error = move || match cambiar.value().get() {
        Some(Err(e)) => Some(mensaje_error(&e)),
        _ => None,
    };

    view! {
        <section class="tarjeta ancha">
            <h1>"Usuarios y roles"</h1>
            <p class="pista">
                "Cada cambio escribe en Redis, emite un domain event y suma en "
                <code>"pokeapi_cambios_rol_total"</code> "."
            </p>
            <Show when=move || error().is_some()>
                <p class="aviso-error">{error}</p>
            </Show>
            <Suspense fallback=|| view! { <p class="cargando">"Cargando…"</p> }>
                {move || {
                    usuarios.get().map(|resultado| match resultado {
                        Err(e) => view! { <p class="aviso-error">{mensaje_error(&e)}</p> }.into_any(),
                        Ok(lista) => {
                            view! {
                                <table class="tabla-usuarios">
                                    <thead>
                                        <tr>
                                            <th>"Usuario"</th>
                                            <th>"Rol"</th>
                                            <th>"Creado"</th>
                                            <th>"Cambiar a"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {lista
                                            .into_iter()
                                            .map(|usuario| {
                                                let nombre = usuario.nombre.clone();
                                                let fecha = usuario
                                                    .creado_en
                                                    .get(0..10)
                                                    .unwrap_or("")
                                                    .to_string();
                                                view! {
                                                    <tr>
                                                        <td>{usuario.nombre.clone()}</td>
                                                        <td>
                                                            <span class=format!(
                                                                "chip rol-{}",
                                                                usuario.rol.to_lowercase(),
                                                            )>{usuario.rol.clone()}</span>
                                                        </td>
                                                        <td class="fecha">{fecha}</td>
                                                        <td class="botones-rol">
                                                            {ROLES
                                                                .iter()
                                                                .filter(|rol| **rol != usuario.rol)
                                                                .map(|rol| {
                                                                    let nombre = nombre.clone();
                                                                    let rol_nuevo = (*rol).to_string();
                                                                    view! {
                                                                        <button
                                                                            class="secundario"
                                                                            on:click=move |_| {
                                                                                cambiar
                                                                                    .dispatch((nombre.clone(), rol_nuevo.clone()));
                                                                            }
                                                                        >
                                                                            {*rol}
                                                                        </button>
                                                                    }
                                                                })
                                                                .collect_view()}
                                                        </td>
                                                    </tr>
                                                }
                                            })
                                            .collect_view()}
                                    </tbody>
                                </table>
                            }
                                .into_any()
                        }
                    })
                }}
            </Suspense>
        </section>
    }
}
