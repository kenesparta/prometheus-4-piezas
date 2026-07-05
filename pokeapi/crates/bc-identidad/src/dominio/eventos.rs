//! Domain Events del BC identidad.
//!
//! Hechos pasados, en participio. El agregado los produce (y, para el ciclo
//! de vida de la sesión, los casos de uso); la capa de aplicación los entrega
//! al publicador, y la infraestructura decide cómo difundirlos (logs,
//! métricas Prometheus, cola…).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::modelo::{NombreUsuario, Rol};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tipo", content = "datos")]
pub enum EventoIdentidad {
    UsuarioRegistrado {
        nombre: NombreUsuario,
        rol: Rol,
        en: DateTime<Utc>,
    },
    RolCambiado {
        nombre: NombreUsuario,
        anterior: Rol,
        nuevo: Rol,
        en: DateTime<Utc>,
    },
    SesionIniciada {
        nombre: NombreUsuario,
        rol: Rol,
        en: DateTime<Utc>,
    },
    /// El nombre llega tal cual lo tecleó quien intentó entrar: puede no
    /// corresponder a un usuario real, por eso es `String` y no el VO.
    LoginFallido { nombre: String, en: DateTime<Utc> },
    SesionCerrada { nombre: NombreUsuario, en: DateTime<Utc> },
}

impl EventoIdentidad {
    pub fn nombre(&self) -> &'static str {
        match self {
            Self::UsuarioRegistrado { .. } => "identidad.usuario_registrado",
            Self::RolCambiado { .. } => "identidad.rol_cambiado",
            Self::SesionIniciada { .. } => "identidad.sesion_iniciada",
            Self::LoginFallido { .. } => "identidad.login_fallido",
            Self::SesionCerrada { .. } => "identidad.sesion_cerrada",
        }
    }
}
