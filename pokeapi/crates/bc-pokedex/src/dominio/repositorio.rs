//! Puertos de persistencia del BC pokedex.
//!
//! Los traits se definen aquí (en el dominio) y su implementación concreta
//! (Redis) vive en el crate binario.

use async_trait::async_trait;
use thiserror::Error;

use super::modelo::{ConsultaRegistrada, FichaPokemon, NombrePokemon};

/// Caché de fichas (patrón cache-aside sobre Redis con TTL).
#[async_trait]
pub trait CacheFichas: Send + Sync {
    async fn obtener(
        &self,
        nombre: &NombrePokemon,
    ) -> Result<Option<FichaPokemon>, ErrorRepositorio>;

    async fn guardar(
        &self,
        ficha: &FichaPokemon,
        ttl_segundos: u64,
    ) -> Result<(), ErrorRepositorio>;
}

/// Bitácora de consultas: lista acotada con las más recientes primero.
#[async_trait]
pub trait RegistroConsultas: Send + Sync {
    async fn agregar(&self, consulta: &ConsultaRegistrada) -> Result<(), ErrorRepositorio>;

    async fn recientes(&self, limite: usize)
    -> Result<Vec<ConsultaRegistrada>, ErrorRepositorio>;

    async fn limpiar(&self) -> Result<(), ErrorRepositorio>;
}

#[derive(Debug, Error)]
pub enum ErrorRepositorio {
    #[error("error de infraestructura: {0}")]
    Infraestructura(String),
}
