//! Domain Events del BC pokedex.
//!
//! Hechos pasados, en participio. Los produce la capa de aplicación al
//! resolver consultas; la infraestructura decide cómo difundirlos (logs
//! estructurados, métricas Prometheus…).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::modelo::OrigenConsulta;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tipo", content = "datos")]
pub enum EventoPokedex {
    PokemonConsultado {
        pokemon: String,
        origen: OrigenConsulta,
        usuario: String,
        rol: String,
        en: DateTime<Utc>,
    },
    ConsultaFallida {
        pokemon: String,
        usuario: String,
        motivo: MotivoFallo,
        en: DateTime<Utc>,
    },
    HistorialLimpiado { usuario: String, en: DateTime<Utc> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MotivoFallo {
    NoEncontrado,
    FuenteNoDisponible,
}

impl MotivoFallo {
    pub fn como_str(&self) -> &'static str {
        match self {
            MotivoFallo::NoEncontrado => "no_encontrado",
            MotivoFallo::FuenteNoDisponible => "fuente_no_disponible",
        }
    }
}

impl EventoPokedex {
    pub fn nombre(&self) -> &'static str {
        match self {
            Self::PokemonConsultado { .. } => "pokedex.pokemon_consultado",
            Self::ConsultaFallida { .. } => "pokedex.consulta_fallida",
            Self::HistorialLimpiado { .. } => "pokedex.historial_limpiado",
        }
    }
}
