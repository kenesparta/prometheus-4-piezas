//! View-models de la UI.
//!
//! Son los únicos tipos que cruzan el cable entre navegador y servidor (los
//! serializan las server functions). Deliberadamente separados de los DTOs
//! de los Bounded Contexts: así el artefacto wasm no arrastra el dominio.

use leptos::prelude::ServerFnError;
use serde::{Deserialize, Serialize};

pub const ROLES: [&str; 3] = ["ADMIN", "EDITOR", "VISITOR"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SesionUi {
    pub nombre: String,
    pub rol: String,
}

impl SesionUi {
    pub fn es_admin(&self) -> bool {
        self.rol == "ADMIN"
    }

    pub fn puede_editar(&self) -> bool {
        matches!(self.rol.as_str(), "ADMIN" | "EDITOR")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FichaUi {
    pub numero: u32,
    pub nombre: String,
    pub tipos: Vec<String>,
    pub estadisticas: Vec<EstadisticaUi>,
    pub altura_dm: u32,
    pub peso_hg: u32,
    pub sprite_url: Option<String>,
    /// `cache` o `api`.
    pub origen: String,
    /// Lo que tardó el servidor en resolver la consulta (se muestra junto al
    /// origen para que se "vea" la diferencia caché vs. API).
    pub duracion_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EstadisticaUi {
    pub nombre: String,
    pub valor: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsultaUi {
    pub usuario: String,
    pub rol: String,
    pub pokemon: String,
    pub origen: String,
    pub exito: bool,
    /// RFC 3339.
    pub en: String,
}

impl ConsultaUi {
    /// `HH:MM:SS` extraído del RFC 3339 (suficiente para la lista).
    pub fn hora(&self) -> String {
        self.en.get(11..19).unwrap_or("").to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsuarioUi {
    pub nombre: String,
    pub rol: String,
    /// RFC 3339.
    pub creado_en: String,
}

/// Mensaje legible de un error de server function (el `ServerError` llega ya
/// en español desde los casos de uso).
pub fn mensaje_error(error: &ServerFnError) -> String {
    match error {
        ServerFnError::ServerError(mensaje) => mensaje.clone(),
        otro => otro.to_string(),
    }
}

/// `mr-mime` → `Mr Mime`.
pub fn formatear_nombre(nombre: &str) -> String {
    nombre
        .split('-')
        .map(|parte| {
            let mut caracteres = parte.chars();
            match caracteres.next() {
                Some(primero) => primero.to_uppercase().chain(caracteres).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
