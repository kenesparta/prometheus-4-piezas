//! Sesión de la petición en curso: cookie → Redis → `SesionActual`.
//!
//! La capa de observabilidad (`metricas::capa_http`) resuelve la sesión una
//! vez por petición y la deja en las extensions; handlers y server functions
//! la leen de ahí sin volver a tocar Redis.

use axum::http::HeaderMap;
use bc_identidad::aplicacion::dto::VistaSesion;

use crate::composicion::Contenedor;

pub const COOKIE_SESION: &str = "pokeapi_sesion";

/// Sesión (o ausencia de ella) de la petición en curso.
#[derive(Debug, Clone)]
pub struct SesionActual(pub Option<VistaSesion>);

impl SesionActual {
    /// Etiqueta de rol para métricas: `ADMIN`, `EDITOR`, `VISITOR` o `anonimo`.
    pub fn etiqueta_rol(&self) -> String {
        self.0.as_ref().map_or_else(|| "anonimo".to_string(), |s| s.rol.clone())
    }

    pub fn usuario(&self) -> String {
        self.0
            .as_ref()
            .map_or_else(|| "anonimo".to_string(), |s| s.nombre_usuario.clone())
    }
}

/// Extrae el token de sesión de la cabecera `Cookie`.
pub fn token_de_cabeceras(cabeceras: &HeaderMap) -> Option<String> {
    let cookies = cabeceras.get(axum::http::header::COOKIE)?.to_str().ok()?;
    cookies.split(';').find_map(|par| {
        let (clave, valor) = par.trim().split_once('=')?;
        (clave == COOKIE_SESION).then(|| valor.to_string())
    })
}

/// Resuelve la sesión contra Redis. Un fallo de infraestructura degrada a
/// "sin sesión" (y queda en las métricas del repositorio), nunca tumba la
/// petición.
pub async fn resolver(contenedor: &Contenedor, cabeceras: &HeaderMap) -> SesionActual {
    let Some(token) = token_de_cabeceras(cabeceras) else {
        return SesionActual(None);
    };
    match contenedor.identidad.validar_sesion.ejecutar(&token).await {
        Ok(sesion) => SesionActual(sesion),
        Err(error) => {
            tracing::warn!(%error, "no se pudo validar la sesión");
            SesionActual(None)
        }
    }
}

/// Cookie de sesión endurecida razonablemente para una demo servida por HTTP
/// plano (sin `Secure`: el LoadBalancer de la charla no termina TLS).
pub fn cookie_sesion(token: &str, ttl_segundos: u64) -> String {
    format!("{COOKIE_SESION}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={ttl_segundos}")
}

pub fn cookie_borrado() -> String {
    format!("{COOKIE_SESION}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0")
}
