//! Capa de aplicación del BC pokedex.
//!
//! Orquesta las consultas: primero el caché, después la fuente externa
//! (PokeAPI) a través del puerto, registra cada consulta en la bitácora y
//! publica los eventos. No contiene detalles de transporte ni de Redis.

pub mod casos_uso;
pub mod dto;
pub mod puertos;
