//! Vista de inicio de sesión (admin/123 o cualquier cuenta registrada).

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

use super::RecursoSesion;
use super::api::IniciarSesion;
use super::modelos::mensaje_error;

#[component]
pub fn PaginaLogin() -> impl IntoView {
    let accion = ServerAction::<IniciarSesion>::new();
    let sesion = expect_context::<RecursoSesion>();
    let navegar = use_navigate();

    Effect::new(move |_| {
        if let Some(Ok(_)) = accion.value().get() {
            sesion.refetch();
            navegar("/dashboard", Default::default());
        }
    });

    let error = move || accion.value().get().and_then(|r| r.err()).map(|e| mensaje_error(&e));

    view! {
        <section class="tarjeta formulario">
            <h1>"Iniciar sesión"</h1>
            <ActionForm action=accion>
                <label>
                    "Usuario"
                    <input type="text" name="nombre" required autocomplete="username"/>
                </label>
                <label>
                    "Password"
                    <input
                        type="password"
                        name="password"
                        required
                        autocomplete="current-password"
                    />
                </label>
                <button type="submit" disabled=move || accion.pending().get()>
                    {move || if accion.pending().get() { "Entrando…" } else { "Entrar" }}
                </button>
            </ActionForm>
            <Show when=move || error().is_some()>
                <p class="aviso-error">{error}</p>
            </Show>
            <p class="pie">"¿Sin cuenta? " <A href="/registro">"Regístrate"</A></p>
        </section>
    }
}
