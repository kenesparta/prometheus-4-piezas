//! Vista de registro: solo nombre de usuario y password.
//!
//! Toda cuenta nueva entra con rol VISITOR; el usuario admin puede promover
//! después desde el panel. El alta inicia sesión en el mismo paso.

use leptos::prelude::*;
use leptos_router::components::{A, Redirect};

use super::RecursoSesion;
use super::api::RegistrarCuenta;
use super::modelos::mensaje_error;

#[component]
pub fn PaginaRegistro() -> impl IntoView {
    let accion = ServerAction::<RegistrarCuenta>::new();
    let sesion = expect_context::<RecursoSesion>();

    // El alta inicia sesión en el mismo paso. Refrescamos la sesión global y
    // dejamos que el redirect reactivo de abajo lleve al dashboard cuando esté
    // cargada (navegar a mano rebotaría a /login: /dashboard vería la sesión
    // aún vieja durante el refetch).
    Effect::new(move |_| {
        if let Some(Ok(_)) = accion.value().get() {
            sesion.refetch();
        }
    });

    let error = move || accion.value().get().and_then(|r| r.err()).map(|e| mensaje_error(&e));

    view! {
        <Suspense fallback=|| ()>
            {move || {
                matches!(sesion.get(), Some(Some(_)))
                    .then(|| view! { <Redirect path="/dashboard"/> })
            }}
        </Suspense>
        <section class="tarjeta formulario">
            <h1>"Crear cuenta"</h1>
            <p class="pista">"Entras como VISITOR; un ADMIN puede cambiarte el rol después."</p>
            <ActionForm action=accion>
                <label>
                    "Usuario"
                    <input
                        type="text"
                        name="nombre"
                        required
                        minlength="3"
                        maxlength="30"
                        autocomplete="username"
                    />
                </label>
                <label>
                    "Password"
                    <input
                        type="password"
                        name="password"
                        required
                        minlength="3"
                        autocomplete="new-password"
                    />
                </label>
                <button type="submit" disabled=move || accion.pending().get()>
                    {move || if accion.pending().get() { "Creando…" } else { "Registrarme" }}
                </button>
            </ActionForm>
            <Show when=move || error().is_some()>
                <p class="aviso-error">{error}</p>
            </Show>
            <p class="pie">"¿Ya tienes cuenta? " <A href="/login">"Inicia sesión"</A></p>
        </section>
    }
}
