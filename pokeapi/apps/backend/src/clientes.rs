//! Clientes HTTP salientes (Anti-Corruption Layer hacia servicios externos).
//!
//! Cuarto tipo de adaptador junto a `http/`, `persistencia/` y `mensajeria/`:
//! implementa los puertos de "fuente externa" de los BCs.

pub mod pokeapi;
