//! Puertos de la capa de aplicación del BC identidad.
//!
//! Los puertos de persistencia viven en `dominio/repositorio.rs`. Aquí van
//! los demás: publicador de eventos, hasher de passwords y generador de
//! tokens de sesión.

use async_trait::async_trait;
use thiserror::Error;

use crate::dominio::eventos::EventoIdentidad;
use crate::dominio::modelo::HashPassword;

#[async_trait]
pub trait PublicadorEventos: Send + Sync {
    async fn publicar(&self, eventos: &[EventoIdentidad]);
}

/// Deriva y verifica hashes de password. La implementación concreta (Argon2)
/// vive en el binario: el dominio solo conoce el hash opaco.
pub trait HasherPassword: Send + Sync {
    /// # Errors
    ///
    /// Devuelve [`ErrorHasher`] si la derivación del hash falla (parámetros
    /// inválidos, fallo de la fuente de aleatoriedad, …).
    fn hashear(&self, password_plano: &str) -> Result<HashPassword, ErrorHasher>;

    fn verificar(&self, password_plano: &str, hash: &HashPassword) -> bool;
}

#[derive(Debug, Error)]
#[error("error al derivar el hash del password: {0}")]
pub struct ErrorHasher(pub String);

/// Genera tokens de sesión imposibles de adivinar.
pub trait GeneradorTokens: Send + Sync {
    fn generar(&self) -> String;
}
