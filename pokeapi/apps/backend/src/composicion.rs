//! Composición / wiring del backend.
//!
//! Aquí se instancian los adaptadores (Redis, PokeAPI, métricas) y se
//! inyectan en los casos de uso de cada Bounded Context. El `Contenedor` se
//! comparte con el router HTTP, la capa de observabilidad y las server
//! functions de Leptos (vía contexto).

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use bc_identidad::aplicacion::casos_uso as identidad_uc;
use bc_identidad::aplicacion::puertos::{GeneradorTokens, HasherPassword};
use bc_identidad::dominio::modelo::{NombreUsuario, Rol, Usuario};
use bc_identidad::dominio::repositorio::{
    ErrorRepositorio, RepositorioSesiones, RepositorioUsuarios,
};
use bc_pokedex::aplicacion::casos_uso as pokedex_uc;
use redis::aio::ConnectionManager;

use crate::clientes::pokeapi::FuentePokemonHttp;
use crate::configuracion::Configuracion;
use crate::mensajeria::identidad_eventos::PublicadorIdentidad;
use crate::mensajeria::pokedex_eventos::PublicadorPokedex;
use crate::metricas::Metricas;
use crate::persistencia::identidad_mongo::RepositorioUsuariosMongo;
use crate::persistencia::identidad_redis::RepositorioSesionesRedis;
use crate::persistencia::pokedex_redis::{CacheFichasRedis, RegistroConsultasRedis};
use crate::seguridad::{GeneradorTokensUuid, HasherArgon2};

/// Contenedor de dependencias: casos de uso ya cableados + métricas + las
/// conexiones a Redis y MongoDB (para el chequeo de salud).
#[derive(Clone)]
pub struct Contenedor {
    pub identidad: CasosIdentidad,
    pub pokedex: CasosPokedex,
    pub metricas: Arc<Metricas>,
    pub redis: ConnectionManager,
    pub mongo: mongodb::Database,
    pub config: Arc<Configuracion>,
}

#[derive(Clone)]
pub struct CasosIdentidad {
    pub registrar: Arc<identidad_uc::RegistrarUsuario>,
    pub iniciar_sesion: Arc<identidad_uc::IniciarSesion>,
    pub validar_sesion: Arc<identidad_uc::ValidarSesion>,
    pub cerrar_sesion: Arc<identidad_uc::CerrarSesion>,
    pub listar_usuarios: Arc<identidad_uc::ListarUsuarios>,
    pub cambiar_rol: Arc<identidad_uc::CambiarRolUsuario>,
}

#[derive(Clone)]
pub struct CasosPokedex {
    pub consultar: Arc<pokedex_uc::ConsultarPokemon>,
    pub historial: Arc<pokedex_uc::VerHistorial>,
    pub limpiar_historial: Arc<pokedex_uc::LimpiarHistorial>,
}

pub async fn componer(config: &Configuracion) -> Result<Contenedor> {
    // ---- Infraestructura compartida -------------------------------------
    let cliente_redis = redis::Client::open(config.redis_url.as_str())
        .context("REDIS_URL no es una URL de Redis válida")?;
    let redis = ConnectionManager::new(cliente_redis)
        .await
        .context("no se pudo conectar a Redis")?;
    tracing::info!("conexión a Redis establecida");

    // MongoDB guarda los usuarios. `with_uri_str` valida la URI (y resuelve el
    // SRV en `mongodb+srv://`); el `ping` fuerza la conexión para fallar rápido
    // al arrancar si la base no está disponible.
    let cliente_mongo = mongodb::Client::with_uri_str(&config.mongodb_uri)
        .await
        .context("MONGODB_URI inválida o no se pudo inicializar el cliente de MongoDB")?;
    let mongo = cliente_mongo.database(&config.mongodb_db);
    mongo
        .run_command(mongodb::bson::doc! { "ping": 1 })
        .await
        .context("no se pudo conectar a MongoDB")?;
    tracing::info!(db = %config.mongodb_db, "conexión a MongoDB establecida");

    let metricas = Arc::new(Metricas::nuevas().context("no se pudieron registrar las métricas")?);

    // ---- BC identidad -----------------------------------------------------
    let usuarios: Arc<dyn RepositorioUsuarios> =
        Arc::new(RepositorioUsuariosMongo::nuevo(&mongo, metricas.clone()));
    let sesiones: Arc<dyn RepositorioSesiones> =
        Arc::new(RepositorioSesionesRedis::nuevo(redis.clone(), metricas.clone()));
    let hasher: Arc<dyn HasherPassword> = Arc::new(HasherArgon2);
    let tokens: Arc<dyn GeneradorTokens> = Arc::new(GeneradorTokensUuid);
    let publicador_identidad = Arc::new(PublicadorIdentidad::nuevo(metricas.clone()));

    let identidad = CasosIdentidad {
        registrar: Arc::new(identidad_uc::RegistrarUsuario::nuevo(
            usuarios.clone(),
            hasher.clone(),
            publicador_identidad.clone(),
        )),
        iniciar_sesion: Arc::new(identidad_uc::IniciarSesion::nuevo(
            usuarios.clone(),
            sesiones.clone(),
            hasher.clone(),
            tokens,
            publicador_identidad.clone(),
            config.sesion_ttl_segundos,
        )),
        validar_sesion: Arc::new(identidad_uc::ValidarSesion::nuevo(
            sesiones.clone(),
            config.sesion_ttl_segundos,
        )),
        cerrar_sesion: Arc::new(identidad_uc::CerrarSesion::nuevo(
            sesiones.clone(),
            publicador_identidad.clone(),
        )),
        listar_usuarios: Arc::new(identidad_uc::ListarUsuarios::nuevo(usuarios.clone())),
        cambiar_rol: Arc::new(identidad_uc::CambiarRolUsuario::nuevo(
            usuarios.clone(),
            publicador_identidad,
        )),
    };

    // ---- BC pokedex ---------------------------------------------------------
    let cache = Arc::new(CacheFichasRedis::nuevo(redis.clone(), metricas.clone()));
    let registro = Arc::new(RegistroConsultasRedis::nuevo(redis.clone(), metricas.clone()));
    let fuente = Arc::new(
        FuentePokemonHttp::nueva(config.pokeapi_url_base.clone(), metricas.clone())
            .context("no se pudo construir el cliente de la PokeAPI")?,
    );
    let publicador_pokedex = Arc::new(PublicadorPokedex::nuevo(metricas.clone()));

    let pokedex = CasosPokedex {
        consultar: Arc::new(pokedex_uc::ConsultarPokemon::nuevo(
            cache,
            fuente,
            registro.clone(),
            publicador_pokedex.clone(),
            config.cache_ttl_segundos,
        )),
        historial: Arc::new(pokedex_uc::VerHistorial::nuevo(registro.clone())),
        limpiar_historial: Arc::new(pokedex_uc::LimpiarHistorial::nuevo(
            registro,
            publicador_pokedex,
        )),
    };

    // ---- Arranque -----------------------------------------------------------
    sembrar_admin(&usuarios, &hasher, config).await?;
    lanzar_medidor_periodico(sesiones, usuarios, redis.clone(), mongo.clone(), metricas.clone());

    Ok(Contenedor {
        identidad,
        pokedex,
        metricas,
        redis,
        mongo,
        config: Arc::new(config.clone()),
    })
}

/// Crea la cuenta `admin` (rol ADMIN) si no existe. No publica eventos: la
/// siembra no es tráfico de negocio y no debe inflar las métricas al
/// reiniciar el pod.
async fn sembrar_admin(
    usuarios: &Arc<dyn RepositorioUsuarios>,
    hasher: &Arc<dyn HasherPassword>,
    config: &Configuracion,
) -> Result<()> {
    let nombre = NombreUsuario::nuevo("admin")?;
    let hash = hasher.hashear(&config.admin_password)?;
    let admin = Usuario::registrar_con_rol(nombre, hash, Rol::Admin);

    match usuarios.guardar_nuevo(&admin).await {
        Ok(()) => tracing::info!("usuario admin creado con rol ADMIN"),
        Err(ErrorRepositorio::YaExiste) => tracing::debug!("el usuario admin ya existía"),
        Err(error) => return Err(error).context("no se pudo sembrar el usuario admin"),
    }
    Ok(())
}

/// Tarea de fondo que refresca cada 15 s los gauges (sesiones activas,
/// usuarios por rol, disponibilidad de Redis y de MongoDB).
fn lanzar_medidor_periodico(
    sesiones: Arc<dyn RepositorioSesiones>,
    usuarios: Arc<dyn RepositorioUsuarios>,
    redis: ConnectionManager,
    mongo: mongodb::Database,
    metricas: Arc<Metricas>,
) {
    tokio::spawn(async move {
        let mut intervalo = tokio::time::interval(Duration::from_secs(15));
        loop {
            intervalo.tick().await;

            match sesiones.contar_activas().await {
                Ok(vivas) => metricas.sesiones_activas.set(vivas as i64),
                Err(error) => tracing::warn!(%error, "no se pudieron contar las sesiones"),
            }

            match usuarios.listar().await {
                Ok(lista) => {
                    for rol in Rol::TODOS {
                        let cuantos = lista.iter().filter(|u| u.rol() == rol).count();
                        metricas
                            .usuarios_por_rol
                            .with_label_values(&[rol.como_str()])
                            .set(cuantos as i64);
                    }
                }
                Err(error) => tracing::warn!(%error, "no se pudieron listar los usuarios"),
            }

            let mut con = redis.clone();
            let pong: redis::RedisResult<String> =
                redis::cmd("PING").query_async(&mut con).await;
            metricas.redis_disponible.set(i64::from(pong.is_ok()));

            let ping_mongo = mongo.run_command(mongodb::bson::doc! { "ping": 1 }).await;
            metricas.mongodb_disponible.set(i64::from(ping_mongo.is_ok()));
        }
    });
}
