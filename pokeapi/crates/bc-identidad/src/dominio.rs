//! Capa de dominio del BC identidad.
//!
//! Aquí viven los conceptos del negocio: el agregado `Usuario`, los value
//! objects (`NombreUsuario`, `Rol`, `HashPassword`), la entidad `Sesion`,
//! los Domain Events y los puertos hacia la infraestructura. Nada en este
//! módulo debe conocer HTTP, Redis ni runtime.

pub mod errores;
pub mod eventos;
pub mod modelo;
pub mod repositorio;
