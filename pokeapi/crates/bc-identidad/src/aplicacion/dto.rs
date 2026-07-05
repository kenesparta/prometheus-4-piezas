//! Commands y Views del BC identidad.
//!
//! - `*Cmd`: datos de entrada para un caso de uso. Sin lógica.
//! - `Vista*`: proyección de salida. Sin lógica.
//!
//! No exponen tipos del dominio: el caso de uso traduce entre estos DTOs y
//! los Value Objects / Aggregates internos. Así, cambiar el dominio no
//! rompe el contrato con los adaptadores del binario.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct RegistrarUsuarioCmd {
    pub nombre: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IniciarSesionCmd {
    pub nombre: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CambiarRolCmd {
    /// Rol de quien solicita el cambio (sale de su sesión validada, nunca del
    /// formulario). Solo ADMIN puede cambiar roles.
    pub rol_solicitante: String,
    pub nombre_objetivo: String,
    pub nuevo_rol: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VistaUsuario {
    pub nombre: String,
    pub rol: String,
    /// RFC 3339, listo para mostrar o re-parsear en el borde.
    pub creado_en: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VistaSesion {
    pub token: String,
    pub nombre_usuario: String,
    pub rol: String,
}
