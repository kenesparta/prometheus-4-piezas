//! Repositorio Redis de **sesiones** del BC identidad.
//!
//! Los usuarios viven en MongoDB (ver [`super::identidad_mongo`]); en Redis
//! solo queda lo efímero de la sesión. Esquema de claves:
//! - `pokeapi:sesion:{token}` — STRING JSON de la sesión, con TTL.
//! - `pokeapi:sesiones`       — ZSET token → timestamp de expiración
//!   (permite contar sesiones vivas sin recorrer claves).

use std::sync::Arc;

use async_trait::async_trait;
use bc_identidad::dominio::modelo::Sesion;
use bc_identidad::dominio::repositorio::{ErrorRepositorio, RepositorioSesiones};
use chrono::Utc;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;

use crate::metricas::Metricas;

const CLAVE_ZSET_SESIONES: &str = "pokeapi:sesiones";

fn clave_sesion(token: &str) -> String {
    format!("pokeapi:sesion:{token}")
}

/// Contabiliza la operación en las métricas y traduce el error de Redis al
/// error del puerto.
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
// RepositorioSesionesRedis
// ============================================================================

pub struct RepositorioSesionesRedis {
    redis: ConnectionManager,
    metricas: Arc<Metricas>,
}

impl RepositorioSesionesRedis {
    pub fn nuevo(redis: ConnectionManager, metricas: Arc<Metricas>) -> Self {
        Self { redis, metricas }
    }
}

#[async_trait]
impl RepositorioSesiones for RepositorioSesionesRedis {
    async fn guardar(&self, sesion: &Sesion, ttl_segundos: u64) -> Result<(), ErrorRepositorio> {
        let cuerpo = serde_json::to_string(sesion)
            .map_err(|e| ErrorRepositorio::Infraestructura(e.to_string()))?;
        let expira_en = Utc::now().timestamp() + ttl_segundos as i64;

        let mut con = self.redis.clone();
        mapear(
            con.set_ex::<_, _, ()>(clave_sesion(sesion.token()), cuerpo, ttl_segundos).await,
            &self.metricas,
            "set_ex",
        )?;
        mapear(
            con.zadd::<_, _, _, ()>(CLAVE_ZSET_SESIONES, sesion.token(), expira_en).await,
            &self.metricas,
            "zadd",
        )
    }

    async fn por_token(
        &self,
        token: &str,
        ttl_segundos: u64,
    ) -> Result<Option<Sesion>, ErrorRepositorio> {
        let mut con = self.redis.clone();
        let cuerpo: Option<String> =
            mapear(con.get(clave_sesion(token)).await, &self.metricas, "get")?;
        let Some(cuerpo) = cuerpo else {
            return Ok(None);
        };

        let Ok(sesion) = serde_json::from_str::<Sesion>(&cuerpo) else {
            tracing::warn!("sesión corrupta en Redis; se descarta");
            return Ok(None);
        };

        // Sesión deslizante: cada uso renueva el TTL y la fecha en el zset.
        let expira_en = Utc::now().timestamp() + ttl_segundos as i64;
        mapear(
            con.expire::<_, ()>(clave_sesion(token), ttl_segundos as i64).await,
            &self.metricas,
            "expire",
        )?;
        mapear(
            con.zadd::<_, _, _, ()>(CLAVE_ZSET_SESIONES, token, expira_en).await,
            &self.metricas,
            "zadd",
        )?;
        Ok(Some(sesion))
    }

    async fn eliminar(&self, token: &str) -> Result<(), ErrorRepositorio> {
        let mut con = self.redis.clone();
        mapear(con.del::<_, ()>(clave_sesion(token)).await, &self.metricas, "del")?;
        mapear(
            con.zrem::<_, _, ()>(CLAVE_ZSET_SESIONES, token).await,
            &self.metricas,
            "zrem",
        )
    }

    async fn contar_activas(&self) -> Result<u64, ErrorRepositorio> {
        let mut con = self.redis.clone();
        let ahora = Utc::now().timestamp();
        // Purga los tokens ya expirados y cuenta el resto.
        mapear(
            con.zrembyscore::<_, _, _, ()>(CLAVE_ZSET_SESIONES, "-inf", ahora).await,
            &self.metricas,
            "zrembyscore",
        )?;
        let vivas: u64 = mapear(con.zcard(CLAVE_ZSET_SESIONES).await, &self.metricas, "zcard")?;
        Ok(vivas)
    }
}
