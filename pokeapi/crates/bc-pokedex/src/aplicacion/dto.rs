//! Commands y Views del BC pokedex.
//!
//! - `*Cmd`: datos de entrada para un caso de uso. Sin lógica.
//! - `Vista*`: proyección de salida. Sin lógica.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ConsultarPokemonCmd {
    pub nombre: String,
    /// Quién consulta (nombre de usuario o `anonimo`). Solo para la bitácora
    /// y las métricas; la autorización ocurre en el borde.
    pub usuario: String,
    /// Rol plano de quien consulta (`ADMIN`, `EDITOR`, `VISITOR`, `anonimo`).
    pub rol: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VistaFicha {
    pub numero: u32,
    pub nombre: String,
    pub tipos: Vec<String>,
    pub estadisticas: Vec<VistaEstadistica>,
    pub altura_dm: u32,
    pub peso_hg: u32,
    pub sprite_url: Option<String>,
    /// De dónde salió la respuesta: `cache` o `api`.
    pub origen: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VistaEstadistica {
    pub nombre: String,
    pub valor: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct VistaConsulta {
    pub usuario: String,
    pub rol: String,
    pub pokemon: String,
    pub origen: String,
    pub exito: bool,
    /// RFC 3339, listo para mostrar o re-parsear en el borde.
    pub en: String,
}
