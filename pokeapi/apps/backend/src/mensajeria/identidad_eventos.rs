//! Publicador de eventos del BC identidad: logs estructurados + métricas.

use std::sync::Arc;

use async_trait::async_trait;
use bc_identidad::aplicacion::puertos::PublicadorEventos;
use bc_identidad::dominio::eventos::EventoIdentidad;

use crate::metricas::Metricas;

pub struct PublicadorIdentidad {
    metricas: Arc<Metricas>,
}

impl PublicadorIdentidad {
    pub fn nuevo(metricas: Arc<Metricas>) -> Self {
        Self { metricas }
    }
}

#[async_trait]
impl PublicadorEventos for PublicadorIdentidad {
    async fn publicar(&self, eventos: &[EventoIdentidad]) {
        for evento in eventos {
            match serde_json::to_string(evento) {
                Ok(payload) => tracing::info!(
                    bc = "identidad",
                    evento = evento.nombre(),
                    %payload,
                    "domain event"
                ),
                Err(error) => tracing::error!(
                    bc = "identidad",
                    %error,
                    "no se pudo serializar el evento"
                ),
            }

            match evento {
                EventoIdentidad::SesionIniciada { .. } => {
                    self.metricas.logins.with_label_values(&["exito"]).inc();
                }
                EventoIdentidad::LoginFallido { .. } => {
                    self.metricas.logins.with_label_values(&["fallo"]).inc();
                }
                EventoIdentidad::UsuarioRegistrado { .. } => {
                    self.metricas.usuarios_registrados.inc();
                }
                EventoIdentidad::RolCambiado { .. } => {
                    self.metricas.cambios_rol.inc();
                }
                EventoIdentidad::SesionCerrada { .. } => {}
            }
        }
    }
}
