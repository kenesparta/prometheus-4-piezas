//! Domain Events del BC identidad.
//!
//! Hechos pasados, en participio. El agregado los produce (y, para el ciclo
//! de vida de la sesión, los casos de uso); la capa de aplicación los entrega
//! al publicador, y la infraestructura decide cómo difundirlos (logs,
//! métricas Prometheus, cola…).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::modelo::{NombreUsuario, Rol};

/// Por qué falló un intento de inicio de sesión.
///
/// El caso de uso conoce el motivo real, pero **no** lo revela en la respuesta
/// (todos los fallos devuelven el mismo error genérico, para no permitir
/// enumerar usuarios). El motivo solo viaja en el evento: alimenta los logs y
/// la métrica `pokeapi_login_errores_total`, visibles únicamente en `/metrics`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MotivoFallo {
    /// El nombre tecleado no corresponde a ninguna cuenta (o está mal formado).
    UsuarioNoExiste,
    /// La cuenta existe pero el password no coincide.
    PasswordIncorrecto,
}

impl MotivoFallo {
    /// Valor estable para usar como etiqueta de métrica.
    pub fn como_str(&self) -> &'static str {
        match self {
            Self::UsuarioNoExiste => "usuario_no_existe",
            Self::PasswordIncorrecto => "password_incorrecto",
        }
    }
}

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
    /// Se registró un intento de entrar (antes de saber si sale bien o mal).
    /// Cuenta cada intento exactamente una vez para `pokeapi_login_intentos_total`.
    IntentoLogin { nombre: String, en: DateTime<Utc> },
    SesionIniciada {
        nombre: NombreUsuario,
        rol: Rol,
        en: DateTime<Utc>,
    },
    /// El nombre llega tal cual lo tecleó quien intentó entrar: puede no
    /// corresponder a un usuario real, por eso es `String` y no el VO.
    LoginFallido {
        nombre: String,
        motivo: MotivoFallo,
        en: DateTime<Utc>,
    },
    SesionCerrada { nombre: NombreUsuario, en: DateTime<Utc> },
}

impl EventoIdentidad {
    pub fn nombre(&self) -> &'static str {
        match self {
            Self::UsuarioRegistrado { .. } => "identidad.usuario_registrado",
            Self::RolCambiado { .. } => "identidad.rol_cambiado",
            Self::IntentoLogin { .. } => "identidad.intento_login",
            Self::SesionIniciada { .. } => "identidad.sesion_iniciada",
            Self::LoginFallido { .. } => "identidad.login_fallido",
            Self::SesionCerrada { .. } => "identidad.sesion_cerrada",
        }
    }
}
