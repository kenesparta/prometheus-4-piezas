//! Publicador de eventos del BC pokedex: logs estructurados + métricas.

use std::sync::Arc;

use async_trait::async_trait;
use bc_pokedex::aplicacion::puertos::PublicadorEventos;
use bc_pokedex::dominio::eventos::EventoPokedex;

use crate::metricas::Metricas;

pub struct PublicadorPokedex {
    metricas: Arc<Metricas>,
}

impl PublicadorPokedex {
    pub fn nuevo(metricas: Arc<Metricas>) -> Self {
        Self { metricas }
    }
}

#[async_trait]
impl PublicadorEventos for PublicadorPokedex {
    async fn publicar(&self, eventos: &[EventoPokedex]) {
        for evento in eventos {
            match serde_json::to_string(evento) {
                Ok(payload) => tracing::info!(
                    bc = "pokedex",
                    evento = evento.nombre(),
                    %payload,
                    "domain event"
                ),
                Err(error) => tracing::error!(
                    bc = "pokedex",
                    %error,
                    "no se pudo serializar el evento"
                ),
            }

            match evento {
                EventoPokedex::PokemonConsultado { origen, .. } => {
                    self.metricas
                        .pokemon_consultas
                        .with_label_values(&[origen.como_str(), "exito"])
                        .inc();
                }
                EventoPokedex::ConsultaFallida { motivo, .. } => {
                    self.metricas
                        .pokemon_consultas
                        .with_label_values(&["api", motivo.como_str()])
                        .inc();
                }
                EventoPokedex::HistorialLimpiado { .. } => {}
            }
        }
    }
}
