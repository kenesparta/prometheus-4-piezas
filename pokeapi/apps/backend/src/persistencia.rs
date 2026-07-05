//! Adaptadores de persistencia sobre Redis (un archivo por Bounded Context).
//!
//! Todo lo que la demo guarda vive en Redis con prefijo `pokeapi:`:
//! usuarios (hash + set índice), sesiones (string JSON con TTL + zset por
//! expiración), caché de fichas (string JSON con TTL) y bitácora de
//! consultas (list acotada).

pub mod identidad_redis;
pub mod pokedex_redis;
