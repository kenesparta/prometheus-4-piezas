//! Server functions: el puente entre la UI Leptos y los casos de uso.
//!
//! Cada función corre en el servidor (la parte cliente es un stub que hace
//! `fetch`); el `Contenedor` llega por contexto y la sesión por las
//! extensions que dejó la capa de observabilidad. Los endpoints son estables
//! (`endpoint = "..."`) para que la etiqueta `ruta` de las métricas sea
//! legible.

use leptos::prelude::*;

use super::modelos::{ConsultaUi, FichaUi, SesionUi, UsuarioUi};

/// Cuántas consultas recientes muestra el dashboard.
#[cfg(feature = "ssr")]
const LIMITE_HISTORIAL: usize = 15;

#[cfg(feature = "ssr")]
mod ssr {
    use bc_identidad::aplicacion::dto::VistaSesion;
    use leptos::prelude::*;

    use crate::composicion::Contenedor;

    pub fn contenedor() -> Result<Contenedor, ServerFnError> {
        use_context::<Contenedor>()
            .ok_or_else(|| ServerFnError::new("el contenedor no está en el contexto"))
    }

    pub async fn sesion() -> Result<Option<VistaSesion>, ServerFnError> {
        let axum::Extension(sesion): axum::Extension<crate::sesion::SesionActual> =
            leptos_axum::extract().await?;
        Ok(sesion.0)
    }

    pub async fn sesion_requerida() -> Result<VistaSesion, ServerFnError> {
        sesion().await?.ok_or_else(|| ServerFnError::new("necesitas iniciar sesión"))
    }

    pub fn poner_cookie(cookie: &str) {
        let Some(respuesta) = use_context::<leptos_axum::ResponseOptions>() else {
            return;
        };
        if let Ok(valor) = axum::http::HeaderValue::from_str(cookie) {
            respuesta.append_header(axum::http::header::SET_COOKIE, valor);
        }
    }
}

// ============================================================================
// Sesión
// ============================================================================

#[server(endpoint = "obtener_sesion")]
pub async fn obtener_sesion() -> Result<Option<SesionUi>, ServerFnError> {
    let sesion = ssr::sesion().await?;
    Ok(sesion.map(|s| SesionUi { nombre: s.nombre_usuario, rol: s.rol }))
}

#[server(endpoint = "registrar_cuenta")]
pub async fn registrar_cuenta(
    nombre: String,
    password: String,
) -> Result<SesionUi, ServerFnError> {
    use bc_identidad::aplicacion::dto::{IniciarSesionCmd, RegistrarUsuarioCmd};

    let contenedor = ssr::contenedor()?;
    contenedor
        .identidad
        .registrar
        .ejecutar(RegistrarUsuarioCmd { nombre: nombre.clone(), password: password.clone() })
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Alta + sesión en un solo paso: quien se registra entra directo.
    let vista = contenedor
        .identidad
        .iniciar_sesion
        .ejecutar(IniciarSesionCmd { nombre, password })
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    ssr::poner_cookie(&crate::sesion::cookie_sesion(
        &vista.token,
        contenedor.config.sesion_ttl_segundos,
    ));
    Ok(SesionUi { nombre: vista.nombre_usuario, rol: vista.rol })
}

#[server(endpoint = "iniciar_sesion")]
pub async fn iniciar_sesion(nombre: String, password: String) -> Result<SesionUi, ServerFnError> {
    use bc_identidad::aplicacion::dto::IniciarSesionCmd;

    let contenedor = ssr::contenedor()?;
    let vista = contenedor
        .identidad
        .iniciar_sesion
        .ejecutar(IniciarSesionCmd { nombre, password })
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    ssr::poner_cookie(&crate::sesion::cookie_sesion(
        &vista.token,
        contenedor.config.sesion_ttl_segundos,
    ));
    Ok(SesionUi { nombre: vista.nombre_usuario, rol: vista.rol })
}

#[server(endpoint = "cerrar_sesion")]
pub async fn cerrar_sesion() -> Result<(), ServerFnError> {
    let contenedor = ssr::contenedor()?;
    if let Some(sesion) = ssr::sesion().await? {
        contenedor
            .identidad
            .cerrar_sesion
            .ejecutar(&sesion.token)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
    }
    ssr::poner_cookie(&crate::sesion::cookie_borrado());
    Ok(())
}

// ============================================================================
// Pokedex
// ============================================================================

#[server(endpoint = "consultar_pokemon")]
pub async fn consultar_pokemon(nombre: String) -> Result<FichaUi, ServerFnError> {
    use bc_pokedex::aplicacion::dto::ConsultarPokemonCmd;

    let contenedor = ssr::contenedor()?;
    let sesion = ssr::sesion_requerida().await?;

    let inicio = std::time::Instant::now();
    let vista = contenedor
        .pokedex
        .consultar
        .ejecutar(ConsultarPokemonCmd {
            nombre,
            usuario: sesion.nombre_usuario,
            rol: sesion.rol,
        })
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(FichaUi {
        numero: vista.numero,
        nombre: vista.nombre,
        tipos: vista.tipos,
        estadisticas: vista
            .estadisticas
            .into_iter()
            .map(|e| super::modelos::EstadisticaUi { nombre: e.nombre, valor: e.valor })
            .collect(),
        altura_dm: vista.altura_dm,
        peso_hg: vista.peso_hg,
        sprite_url: vista.sprite_url,
        origen: vista.origen,
        duracion_ms: u64::try_from(inicio.elapsed().as_millis()).unwrap_or(u64::MAX),
    })
}

#[server(endpoint = "historial_reciente")]
pub async fn historial_reciente() -> Result<Vec<ConsultaUi>, ServerFnError> {
    let contenedor = ssr::contenedor()?;
    ssr::sesion_requerida().await?;

    let consultas = contenedor
        .pokedex
        .historial
        .ejecutar(LIMITE_HISTORIAL)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(consultas
        .into_iter()
        .map(|c| ConsultaUi {
            usuario: c.usuario,
            rol: c.rol,
            pokemon: c.pokemon,
            origen: c.origen,
            exito: c.exito,
            en: c.en,
        })
        .collect())
}

#[server(endpoint = "limpiar_historial")]
pub async fn limpiar_historial() -> Result<(), ServerFnError> {
    let contenedor = ssr::contenedor()?;
    let sesion = ssr::sesion_requerida().await?;
    // Política transversal: limpiar es cosa de EDITOR o ADMIN.
    if !matches!(sesion.rol.as_str(), "ADMIN" | "EDITOR") {
        return Err(ServerFnError::new("solo EDITOR o ADMIN pueden limpiar el historial"));
    }
    contenedor
        .pokedex
        .limpiar_historial
        .ejecutar(sesion.nombre_usuario)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

// ============================================================================
// Administración
// ============================================================================

#[server(endpoint = "listar_usuarios")]
pub async fn listar_usuarios() -> Result<Vec<UsuarioUi>, ServerFnError> {
    let contenedor = ssr::contenedor()?;
    let sesion = ssr::sesion_requerida().await?;

    let usuarios = contenedor
        .identidad
        .listar_usuarios
        .ejecutar(&sesion.rol)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(usuarios
        .into_iter()
        .map(|u| UsuarioUi { nombre: u.nombre, rol: u.rol, creado_en: u.creado_en })
        .collect())
}

#[server(endpoint = "cambiar_rol")]
pub async fn cambiar_rol(nombre: String, nuevo_rol: String) -> Result<UsuarioUi, ServerFnError> {
    use bc_identidad::aplicacion::dto::CambiarRolCmd;

    let contenedor = ssr::contenedor()?;
    let sesion = ssr::sesion_requerida().await?;

    let vista = contenedor
        .identidad
        .cambiar_rol
        .ejecutar(CambiarRolCmd {
            rol_solicitante: sesion.rol,
            nombre_objetivo: nombre,
            nuevo_rol,
        })
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(UsuarioUi { nombre: vista.nombre, rol: vista.rol, creado_en: vista.creado_en })
}
