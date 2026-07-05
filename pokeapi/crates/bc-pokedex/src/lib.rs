//! Bounded Context: pokedex
//!
//! Consultas de pokémon (con caché e historial) sobre la PokeAPI pública.
//! Solo contiene las capas `dominio` y `aplicacion`. No conoce HTTP, Redis
//! ni runtime: los adaptadores viven en el crate binario.

pub mod aplicacion;
pub mod dominio;
