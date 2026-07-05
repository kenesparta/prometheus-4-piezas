//! Puertos de persistencia del BC identidad.
//!
//! Los traits se definen aquí (en el dominio) y su implementación concreta
//! (Redis, en memoria, etc.) vive en el crate binario.

use async_trait::async_trait;
use thiserror::Error;

use super::modelo::{NombreUsuario, Sesion, Usuario};

#[async_trait]
pub trait RepositorioUsuarios: Send + Sync {
    async fn por_nombre(
        &self,
        nombre: &NombreUsuario,
    ) -> Result<Option<Usuario>, ErrorRepositorio>;

    /// Persiste un usuario nuevo. Debe fallar con [`ErrorRepositorio::YaExiste`]
    /// si el nombre ya está tomado (de forma atómica, sin carrera).
    async fn guardar_nuevo(&self, usuario: &Usuario) -> Result<(), ErrorRepositorio>;

    /// Actualiza un usuario existente (p. ej. tras un cambio de rol).
    async fn guardar(&self, usuario: &Usuario) -> Result<(), ErrorRepositorio>;

    async fn listar(&self) -> Result<Vec<Usuario>, ErrorRepositorio>;
}

#[async_trait]
pub trait RepositorioSesiones: Send + Sync {
    async fn guardar(&self, sesion: &Sesion, ttl_segundos: u64) -> Result<(), ErrorRepositorio>;

    /// Recupera la sesión por token y extiende su TTL (sesión deslizante).
    async fn por_token(
        &self,
        token: &str,
        ttl_segundos: u64,
    ) -> Result<Option<Sesion>, ErrorRepositorio>;

    async fn eliminar(&self, token: &str) -> Result<(), ErrorRepositorio>;

    /// Número de sesiones vivas (para la métrica `pokeapi_sesiones_activas`).
    async fn contar_activas(&self) -> Result<u64, ErrorRepositorio>;
}

#[derive(Debug, Error)]
pub enum ErrorRepositorio {
    #[error("ya existe una entidad con esa clave")]
    YaExiste,

    #[error("error de infraestructura: {0}")]
    Infraestructura(String),
}
