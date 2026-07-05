//! Repositorios Redis para el BC identidad.
//!
//! Esquema de claves:
//! - `pokeapi:usuarios`            — SET con los nombres (índice + candado de unicidad).
//! - `pokeapi:usuario:{nombre}`    — HASH con los campos del agregado.
//! - `pokeapi:sesion:{token}`      — STRING JSON de la sesión, con TTL.
//! - `pokeapi:sesiones`            — ZSET token → timestamp de expiración
//!   (permite contar sesiones vivas sin recorrer claves).

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use bc_identidad::dominio::modelo::{
    HashPassword, NombreUsuario, Rol, Sesion, Usuario,
};
use bc_identidad::dominio::repositorio::{
    ErrorRepositorio, RepositorioSesiones, RepositorioUsuarios,
};
use chrono::{DateTime, Utc};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;

use crate::metricas::Metricas;

const CLAVE_INDICE_USUARIOS: &str = "pokeapi:usuarios";
const CLAVE_ZSET_SESIONES: &str = "pokeapi:sesiones";

fn clave_usuario(nombre: &NombreUsuario) -> String {
    format!("pokeapi:usuario:{}", nombre.como_str())
}

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
// RepositorioUsuariosRedis
// ============================================================================

pub struct RepositorioUsuariosRedis {
    redis: ConnectionManager,
    metricas: Arc<Metricas>,
}

impl RepositorioUsuariosRedis {
    pub fn nuevo(redis: ConnectionManager, metricas: Arc<Metricas>) -> Self {
        Self { redis, metricas }
    }

    fn campos(usuario: &Usuario) -> [(&'static str, String); 4] {
        [
            ("hash_password", usuario.hash_password().como_str().to_string()),
            ("rol", usuario.rol().como_str().to_string()),
            ("creado_en", usuario.creado_en().to_rfc3339()),
            ("version", usuario.version().to_string()),
        ]
    }

    fn hidratar(
        nombre: &NombreUsuario,
        campos: &HashMap<String, String>,
    ) -> Result<Usuario, ErrorRepositorio> {
        let corrupto =
            |detalle: &str| ErrorRepositorio::Infraestructura(format!("registro corrupto: {detalle}"));

        let hash = campos.get("hash_password").ok_or_else(|| corrupto("sin hash_password"))?;
        let rol = Rol::desde_str(campos.get("rol").ok_or_else(|| corrupto("sin rol"))?)
            .map_err(|e| corrupto(&e.to_string()))?;
        let creado_en = campos
            .get("creado_en")
            .and_then(|v| DateTime::parse_from_rfc3339(v).ok())
            .map(|f| f.with_timezone(&Utc))
            .ok_or_else(|| corrupto("creado_en inválido"))?;
        let version = campos.get("version").and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);

        Ok(Usuario::hidratar(
            nombre.clone(),
            HashPassword::desde_cadena(hash.clone()),
            rol,
            creado_en,
            version,
        ))
    }
}

#[async_trait]
impl RepositorioUsuarios for RepositorioUsuariosRedis {
    async fn por_nombre(
        &self,
        nombre: &NombreUsuario,
    ) -> Result<Option<Usuario>, ErrorRepositorio> {
        let mut con = self.redis.clone();
        let campos: HashMap<String, String> =
            mapear(con.hgetall(clave_usuario(nombre)).await, &self.metricas, "hgetall")?;
        if campos.is_empty() {
            return Ok(None);
        }
        Self::hidratar(nombre, &campos).map(Some)
    }

    async fn guardar_nuevo(&self, usuario: &Usuario) -> Result<(), ErrorRepositorio> {
        let mut con = self.redis.clone();
        // SADD devuelve 1 solo para miembros nuevos: es el candado atómico
        // contra registros duplicados.
        let nuevos: i64 = mapear(
            con.sadd(CLAVE_INDICE_USUARIOS, usuario.nombre().como_str()).await,
            &self.metricas,
            "sadd",
        )?;
        if nuevos == 0 {
            return Err(ErrorRepositorio::YaExiste);
        }
        mapear(
            con.hset_multiple::<_, _, _, ()>(
                clave_usuario(usuario.nombre()),
                &Self::campos(usuario),
            )
            .await,
            &self.metricas,
            "hset",
        )
    }

    async fn guardar(&self, usuario: &Usuario) -> Result<(), ErrorRepositorio> {
        let mut con = self.redis.clone();
        mapear(
            con.hset_multiple::<_, _, _, ()>(
                clave_usuario(usuario.nombre()),
                &Self::campos(usuario),
            )
            .await,
            &self.metricas,
            "hset",
        )
    }

    async fn listar(&self) -> Result<Vec<Usuario>, ErrorRepositorio> {
        let mut con = self.redis.clone();
        let nombres: Vec<String> =
            mapear(con.smembers(CLAVE_INDICE_USUARIOS).await, &self.metricas, "smembers")?;

        let mut usuarios = Vec::with_capacity(nombres.len());
        for cadena in nombres {
            let Ok(nombre) = NombreUsuario::nuevo(cadena.clone()) else {
                tracing::warn!(nombre = %cadena, "nombre inválido en el índice de usuarios");
                continue;
            };
            if let Some(usuario) = self.por_nombre(&nombre).await? {
                usuarios.push(usuario);
            }
        }
        Ok(usuarios)
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
