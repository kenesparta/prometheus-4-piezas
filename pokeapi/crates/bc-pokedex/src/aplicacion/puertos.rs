//! Puertos de la capa de aplicación del BC pokedex.
//!
//! Los puertos de persistencia (caché y bitácora) viven en
//! `dominio/repositorio.rs`. Aquí van los demás: la fuente externa de
//! pokémon (Anti-Corruption Layer sobre la PokeAPI) y el publicador de
//! eventos.

use async_trait::async_trait;
use thiserror::Error;

use crate::dominio::eventos::EventoPokedex;
use crate::dominio::modelo::{FichaPokemon, NombrePokemon};

/// Fuente externa de fichas. La implementación concreta (cliente HTTP a
/// pokeapi.co) vive en el binario y traduce el modelo externo al nuestro.
#[async_trait]
pub trait FuentePokemon: Send + Sync {
    /// # Errors
    ///
    /// - [`ErrorFuente::NoEncontrado`] si la PokeAPI no conoce ese pokémon.
    /// - [`ErrorFuente::Infraestructura`] ante fallos de red o respuestas
    ///   inesperadas.
    async fn obtener(&self, nombre: &NombrePokemon) -> Result<FichaPokemon, ErrorFuente>;
}

#[derive(Debug, Error)]
pub enum ErrorFuente {
    #[error("la PokeAPI no conoce ese pokémon")]
    NoEncontrado,

    #[error("la fuente externa no está disponible: {0}")]
    Infraestructura(String),
}

#[async_trait]
pub trait PublicadorEventos: Send + Sync {
    async fn publicar(&self, eventos: &[EventoPokedex]);
}
