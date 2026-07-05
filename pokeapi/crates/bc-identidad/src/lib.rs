//! Bounded Context: identidad
//!
//! Usuarios, credenciales, roles (ADMIN / EDITOR / VISITOR) y sesiones.
//! Solo contiene las capas `dominio` y `aplicacion`. No conoce HTTP, Redis
//! ni runtime: los adaptadores viven en el crate binario.

pub mod aplicacion;
pub mod dominio;
