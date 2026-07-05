//! Adaptadores Redis para el BC pokedex.
//!
//! Esquema de claves:
//! - `pokeapi:pokemon:{nombre}` — STRING JSON de la ficha, con TTL (caché).
//! - `pokeapi:consultas`        — LIST JSON con las consultas más recientes
//!   primero, acotada a [`MAX_CONSULTAS`] entradas.

use std::sync::Arc;

use async_trait::async_trait;
use bc_pokedex::dominio::modelo::{ConsultaRegistrada, FichaPokemon, NombrePokemon};
use bc_pokedex::dominio::repositorio::{CacheFichas, ErrorRepositorio, RegistroConsultas};
use redis::AsyncCommands;
use redis::aio::ConnectionManager;

use crate::metricas::Metricas;

const CLAVE_CONSULTAS: &str = "pokeapi:consultas";
const MAX_CONSULTAS: isize = 200;

fn clave_pokemon(nombre: &str) -> String {
    format!("pokeapi:pokemon:{nombre}")
}

fn mapear<T>(
    resultado: redis::RedisResult<T>,
    metricas: &Metricas,
    operacion: &'static str,
) -> Result<T, ErrorRepositorio> {
    match resultado {
        Ok(valor) => {
            metricas.redis_operaciones.with_label_values(&[operacion, "ok"]).inc();
            Ok(valor)
        }
        Err(error) => {
            metricas.redis_operaciones.with_label_values(&[operacion, "error"]).inc();
            Err(ErrorRepositorio::Infraestructura(error.to_string()))
        }
    }
}

// ============================================================================
// CacheFichasRedis
// ============================================================================

pub struct CacheFichasRedis {
    redis: ConnectionManager,
    metricas: Arc<Metricas>,
}

impl CacheFichasRedis {
    pub fn nuevo(redis: ConnectionManager, metricas: Arc<Metricas>) -> Self {
        Self { redis, metricas }
    }
}

#[async_trait]
impl CacheFichas for CacheFichasRedis {
    async fn obtener(
        &self,
        nombre: &NombrePokemon,
    ) -> Result<Option<FichaPokemon>, ErrorRepositorio> {
        let mut con = self.redis.clone();
        let cuerpo: Option<String> =
            mapear(con.get(clave_pokemon(nombre.como_str())).await, &self.metricas, "get")?;
        let Some(cuerpo) = cuerpo else {
            return Ok(None);
        };
        match serde_json::from_str::<FichaPokemon>(&cuerpo) {
            Ok(ficha) => Ok(Some(ficha)),
            Err(error) => {
                // Entrada corrupta: se trata como miss y se regenerará.
                tracing::warn!(%error, pokemon = %nombre, "ficha corrupta en caché");
                Ok(None)
            }
        }
    }

    async fn guardar(
        &self,
        ficha: &FichaPokemon,
        ttl_segundos: u64,
    ) -> Result<(), ErrorRepositorio> {
        let cuerpo = serde_json::to_string(ficha)
            .map_err(|e| ErrorRepositorio::Infraestructura(e.to_string()))?;
        let mut con = self.redis.clone();
        mapear(
            con.set_ex::<_, _, ()>(clave_pokemon(&ficha.nombre), cuerpo, ttl_segundos).await,
            &self.metricas,
            "set_ex",
        )
    }
}

// ============================================================================
// RegistroConsultasRedis
// ============================================================================

pub struct RegistroConsultasRedis {
    redis: ConnectionManager,
    metricas: Arc<Metricas>,
}

impl RegistroConsultasRedis {
    pub fn nuevo(redis: ConnectionManager, metricas: Arc<Metricas>) -> Self {
        Self { redis, metricas }
    }
}

#[async_trait]
impl RegistroConsultas for RegistroConsultasRedis {
    async fn agregar(&self, consulta: &ConsultaRegistrada) -> Result<(), ErrorRepositorio> {
        let cuerpo = serde_json::to_string(consulta)
            .map_err(|e| ErrorRepositorio::Infraestructura(e.to_string()))?;
        let mut con = self.redis.clone();
        mapear(
            con.lpush::<_, _, ()>(CLAVE_CONSULTAS, cuerpo).await,
            &self.metricas,
            "lpush",
        )?;
        mapear(
            con.ltrim::<_, ()>(CLAVE_CONSULTAS, 0, MAX_CONSULTAS - 1).await,
            &self.metricas,
            "ltrim",
        )
    }

    async fn recientes(
        &self,
        limite: usize,
    ) -> Result<Vec<ConsultaRegistrada>, ErrorRepositorio> {
        let mut con = self.redis.clone();
        let filas: Vec<String> = mapear(
            con.lrange(CLAVE_CONSULTAS, 0, limite.saturating_sub(1) as isize).await,
            &self.metricas,
            "lrange",
        )?;
        Ok(filas
            .iter()
            .filter_map(|fila| serde_json::from_str::<ConsultaRegistrada>(fila).ok())
            .collect())
    }

    async fn limpiar(&self) -> Result<(), ErrorRepositorio> {
        let mut con = self.redis.clone();
        mapear(con.del::<_, ()>(CLAVE_CONSULTAS).await, &self.metricas, "del")
    }
}
