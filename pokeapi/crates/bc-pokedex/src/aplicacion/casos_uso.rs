//! Casos de uso del BC pokedex.
//!
//! `ConsultarPokemon` implementa cache-aside: primero Redis, después la
//! PokeAPI; cada consulta queda en la bitácora y produce un evento. Un fallo
//! del caché o de la bitácora degrada la experiencia (se ignora) pero nunca
//! tumba la consulta: la fuente de verdad del error es la fuente externa.

use std::sync::Arc;

use chrono::Utc;
use shared_kernel::ErrorDominio;
use thiserror::Error;

use super::dto::{ConsultarPokemonCmd, VistaConsulta, VistaEstadistica, VistaFicha};
use super::puertos::{ErrorFuente, FuentePokemon, PublicadorEventos};
use crate::dominio::eventos::{EventoPokedex, MotivoFallo};
use crate::dominio::modelo::{
    ConsultaRegistrada, FichaPokemon, NombrePokemon, OrigenConsulta,
};
use crate::dominio::repositorio::{CacheFichas, ErrorRepositorio, RegistroConsultas};

#[derive(Debug, Error)]
pub enum ErrorCasoUso {
    #[error(transparent)]
    Dominio(#[from] ErrorDominio),

    #[error(transparent)]
    Repositorio(#[from] ErrorRepositorio),

    #[error("la PokeAPI no conoce ese pokémon")]
    NoEncontrado,

    #[error("la PokeAPI no está disponible: {0}")]
    Fuente(String),
}

fn vista_ficha(ficha: &FichaPokemon, origen: OrigenConsulta) -> VistaFicha {
    VistaFicha {
        numero: ficha.numero,
        nombre: ficha.nombre.clone(),
        tipos: ficha.tipos.clone(),
        estadisticas: ficha
            .estadisticas
            .iter()
            .map(|e| VistaEstadistica { nombre: e.nombre.clone(), valor: e.valor })
            .collect(),
        altura_dm: ficha.altura_dm,
        peso_hg: ficha.peso_hg,
        sprite_url: ficha.sprite_url.clone(),
        origen: origen.como_str().to_string(),
    }
}

fn vista_consulta(consulta: &ConsultaRegistrada) -> VistaConsulta {
    VistaConsulta {
        usuario: consulta.usuario.clone(),
        rol: consulta.rol.clone(),
        pokemon: consulta.pokemon.clone(),
        origen: consulta.origen.como_str().to_string(),
        exito: consulta.exito,
        en: consulta.en.to_rfc3339(),
    }
}

// ============================================================================
// ConsultarPokemon
// ============================================================================

pub struct ConsultarPokemon {
    cache: Arc<dyn CacheFichas>,
    fuente: Arc<dyn FuentePokemon>,
    registro: Arc<dyn RegistroConsultas>,
    publicador: Arc<dyn PublicadorEventos>,
    cache_ttl_segundos: u64,
}

impl ConsultarPokemon {
    pub fn nuevo(
        cache: Arc<dyn CacheFichas>,
        fuente: Arc<dyn FuentePokemon>,
        registro: Arc<dyn RegistroConsultas>,
        publicador: Arc<dyn PublicadorEventos>,
        cache_ttl_segundos: u64,
    ) -> Self {
        Self { cache, fuente, registro, publicador, cache_ttl_segundos }
    }

    /// # Errors
    ///
    /// - [`ErrorCasoUso::Dominio`] si el nombre no cumple las reglas.
    /// - [`ErrorCasoUso::NoEncontrado`] si la PokeAPI no conoce ese pokémon.
    /// - [`ErrorCasoUso::Fuente`] ante fallos de red con la PokeAPI.
    pub async fn ejecutar(&self, cmd: ConsultarPokemonCmd) -> Result<VistaFicha, ErrorCasoUso> {
        let nombre = NombrePokemon::nuevo(cmd.nombre)?;

        // 1. Caché. Un error aquí degrada a "miss": el adaptador ya dejó
        //    rastro del fallo en sus métricas.
        let en_cache = self.cache.obtener(&nombre).await.unwrap_or_default();
        if let Some(ficha) = en_cache {
            self.registrar(&nombre, &cmd.usuario, &cmd.rol, OrigenConsulta::Cache, true).await;
            return Ok(vista_ficha(&ficha, OrigenConsulta::Cache));
        }

        // 2. Fuente externa (PokeAPI).
        match self.fuente.obtener(&nombre).await {
            Ok(ficha) => {
                // Best effort: si el caché no acepta la escritura, la próxima
                // consulta simplemente volverá a la API.
                let _ = self.cache.guardar(&ficha, self.cache_ttl_segundos).await;
                self.registrar(&nombre, &cmd.usuario, &cmd.rol, OrigenConsulta::Api, true).await;
                Ok(vista_ficha(&ficha, OrigenConsulta::Api))
            }
            Err(ErrorFuente::NoEncontrado) => {
                self.registrar(&nombre, &cmd.usuario, &cmd.rol, OrigenConsulta::Api, false).await;
                self.publicador
                    .publicar(&[EventoPokedex::ConsultaFallida {
                        pokemon: nombre.como_str().to_string(),
                        usuario: cmd.usuario,
                        motivo: MotivoFallo::NoEncontrado,
                        en: Utc::now(),
                    }])
                    .await;
                Err(ErrorCasoUso::NoEncontrado)
            }
            Err(ErrorFuente::Infraestructura(mensaje)) => {
                self.publicador
                    .publicar(&[EventoPokedex::ConsultaFallida {
                        pokemon: nombre.como_str().to_string(),
                        usuario: cmd.usuario,
                        motivo: MotivoFallo::FuenteNoDisponible,
                        en: Utc::now(),
                    }])
                    .await;
                Err(ErrorCasoUso::Fuente(mensaje))
            }
        }
    }

    /// Deja la consulta en la bitácora (best effort) y publica el evento.
    async fn registrar(
        &self,
        nombre: &NombrePokemon,
        usuario: &str,
        rol: &str,
        origen: OrigenConsulta,
        exito: bool,
    ) {
        let consulta = ConsultaRegistrada::nueva(usuario, rol, nombre, origen, exito);
        let _ = self.registro.agregar(&consulta).await;

        if exito {
            self.publicador
                .publicar(&[EventoPokedex::PokemonConsultado {
                    pokemon: consulta.pokemon.clone(),
                    origen,
                    usuario: consulta.usuario.clone(),
                    rol: consulta.rol.clone(),
                    en: consulta.en,
                }])
                .await;
        }
    }
}

// ============================================================================
// VerHistorial
// ============================================================================

pub struct VerHistorial {
    registro: Arc<dyn RegistroConsultas>,
}

impl VerHistorial {
    pub fn nuevo(registro: Arc<dyn RegistroConsultas>) -> Self {
        Self { registro }
    }

    pub async fn ejecutar(&self, limite: usize) -> Result<Vec<VistaConsulta>, ErrorCasoUso> {
        let consultas = self.registro.recientes(limite).await?;
        Ok(consultas.iter().map(vista_consulta).collect())
    }
}

// ============================================================================
// LimpiarHistorial
// ============================================================================

/// Vacía la bitácora de consultas.
///
/// La política de quién puede hacerlo (EDITOR o ADMIN) es transversal a los
/// contextos y la aplica el borde (server function) antes de llegar aquí.
pub struct LimpiarHistorial {
    registro: Arc<dyn RegistroConsultas>,
    publicador: Arc<dyn PublicadorEventos>,
}

impl LimpiarHistorial {
    pub fn nuevo(
        registro: Arc<dyn RegistroConsultas>,
        publicador: Arc<dyn PublicadorEventos>,
    ) -> Self {
        Self { registro, publicador }
    }

    pub async fn ejecutar(&self, usuario: impl Into<String>) -> Result<(), ErrorCasoUso> {
        self.registro.limpiar().await?;
        self.publicador
            .publicar(&[EventoPokedex::HistorialLimpiado {
                usuario: usuario.into(),
                en: Utc::now(),
            }])
            .await;
        Ok(())
    }
}
