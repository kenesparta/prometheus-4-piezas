//! Capa de dominio del BC pokedex.
//!
//! Aquí viven los conceptos del negocio: la ficha de un pokémon tal como la
//! entiende esta aplicación, el registro de consultas y los puertos hacia la
//! infraestructura (caché y bitácora). Nada en este módulo debe conocer HTTP,
//! Redis ni runtime.

pub mod errores;
pub mod eventos;
pub mod modelo;
pub mod repositorio;
