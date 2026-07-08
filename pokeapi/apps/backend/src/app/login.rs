//! Vista de inicio de sesión (admin/123 o cualquier cuenta registrada).

use leptos::prelude::*;
use leptos_router::components::{A, Redirect};

use super::RecursoSesion;
use super::api::IniciarSesion;
use super::modelos::mensaje_error;

#[component]
pub fn PaginaLogin() -> impl IntoView {
    let accion = ServerAction::<IniciarSesion>::new();
    let sesion = expect_context::<RecursoSesion>();

    // Al iniciar sesión con éxito, refresca la sesión global. NO navegamos a
    // mano: `/dashboard` rebotaría a `/login` mientras el recurso de sesión aún
    // conserva su valor viejo ("sin sesión") durante el refetch. En su lugar,
    // el redirect reactivo de abajo lleva al dashboard en cuanto la sesión
    // (recién iniciada o previa) queda cargada.
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
