//! Cliente HTTP a la PokeAPI pública (implementa el puerto `FuentePokemon`).
//!
//! Anti-Corruption Layer: el JSON gigante de la PokeAPI se traduce aquí a la
//! `FichaPokemon` del dominio; ningún otro módulo conoce el modelo externo.

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use bc_pokedex::aplicacion::puertos::{ErrorFuente, FuentePokemon};
use bc_pokedex::dominio::modelo::{Estadistica, FichaPokemon, NombrePokemon};
use serde::Deserialize;

use crate::metricas::Metricas;

pub struct FuentePokemonHttp {
    cliente: reqwest::Client,
    url_base: String,
    metricas: Arc<Metricas>,
}

impl FuentePokemonHttp {
    /// # Errors
    ///
    /// Falla si el cliente HTTP no puede construirse (configuración TLS).
    pub fn nueva(url_base: impl Into<String>, metricas: Arc<Metricas>) -> anyhow::Result<Self> {
        let cliente = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("pokeapi-prometheus-demo/0.1 (charla CNCF)")
            .build()?;
        Ok(Self {
            cliente,
            url_base: url_base.into().trim_end_matches('/').to_string(),
            metricas,
        })
    }
}

#[async_trait]
impl FuentePokemon for FuentePokemonHttp {
    async fn obtener(&self, nombre: &NombrePokemon) -> Result<FichaPokemon, ErrorFuente> {
        let url = format!("{}/pokemon/{}", self.url_base, nombre.como_str());

        let inicio = Instant::now();
        let respuesta = self.cliente.get(&url).send().await;
        self.metricas.upstream_duracion.observe(inicio.elapsed().as_secs_f64());

        let respuesta = match respuesta {
            Ok(r) => r,
            Err(error) => {
                self.metricas.upstream_peticiones.with_label_values(&["error_red"]).inc();
                return Err(ErrorFuente::Infraestructura(error.to_string()));
            }
        };

        let estado = respuesta.status();
        self.metricas.upstream_peticiones.with_label_values(&[estado.as_str()]).inc();

        if estado == reqwest::StatusCode::NOT_FOUND {
            return Err(ErrorFuente::NoEncontrado);
        }
        if !estado.is_success() {
            return Err(ErrorFuente::Infraestructura(format!("HTTP {estado} de la PokeAPI")));
        }

        let cuerpo: RespuestaPokemon = respuesta
            .json()
            .await
            .map_err(|e| ErrorFuente::Infraestructura(format!("JSON inesperado: {e}")))?;
        Ok(cuerpo.en_ficha())
    }
}

// ============================================================================
// Modelo externo (subconjunto del JSON de pokeapi.co)
// ============================================================================

#[derive(Debug, Deserialize)]
struct RespuestaPokemon {
    id: u32,
    name: String,
    height: u32,
    weight: u32,
    types: Vec<EntradaTipo>,
    stats: Vec<EntradaEstadistica>,
    sprites: Sprites,
}

#[derive(Debug, Deserialize)]
struct EntradaTipo {
    #[serde(rename = "type")]
    tipo: Recurso,
}

#[derive(Debug, Deserialize)]
struct EntradaEstadistica {
    base_stat: u32,
    stat: Recurso,
}

#[derive(Debug, Deserialize)]
struct Recurso {
    name: String,
}

#[derive(Debug, Deserialize)]
struct Sprites {
    front_default: Option<String>,
}

impl RespuestaPokemon {
    fn en_ficha(self) -> FichaPokemon {
        FichaPokemon {
            numero: self.id,
            nombre: self.name,
            tipos: self.types.into_iter().map(|t| t.tipo.name).collect(),
            estadisticas: self
                .stats
                .into_iter()
                .map(|e| Estadistica { nombre: e.stat.name, valor: e.base_stat })
                .collect(),
            altura_dm: self.height,
            peso_hg: self.weight,
            sprite_url: self.sprites.front_default,
        }
    }
}
