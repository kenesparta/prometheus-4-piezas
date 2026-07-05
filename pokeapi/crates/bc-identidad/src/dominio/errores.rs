//! Errores propios del dominio del BC identidad.
//!
//! Para los compartidos entre BCs (invariantes genéricas, no encontrado),
//! usa `shared_kernel::ErrorDominio` directamente.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ErrorIdentidad {
    /// Proteger la aplicación de quedarse sin administradores: el último
    /// usuario con rol ADMIN no puede ser degradado.
    #[error("no se puede quitar el rol ADMIN al último administrador")]
    UltimoAdmin,
}
