//! Adaptadores de persistencia (un archivo por tecnología × Bounded Context).
//!
//! - **MongoDB** (`identidad_mongo`) guarda los **usuarios** del BC identidad:
//!   la base de datos "de negocio" de la app (colección `usuarios`).
//! - **Redis** (`identidad_redis`, `pokedex_redis`) guarda lo efímero con
//!   prefijo `pokeapi:`: sesiones (string JSON con TTL + zset por expiración),
//!   caché de fichas (string JSON con TTL) y bitácora de consultas (list acotada).

pub mod identidad_mongo;
pub mod identidad_redis;
pub mod pokedex_redis;
