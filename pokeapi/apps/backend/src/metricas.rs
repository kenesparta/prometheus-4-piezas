//! Métricas Prometheus de la aplicación (pieza 1 de la charla: esta app es
//! un *exporter* que expone `/metrics` por HTTP en texto plano).
//!
//! Familias registradas (prefijo `pokeapi_`):
//!
//! | Métrica                              | Tipo      | Etiquetas                         |
//! |--------------------------------------|-----------|-----------------------------------|
//! | `pokeapi_http_peticiones_total`      | counter   | metodo, ruta, estado, rol         |
//! | `pokeapi_http_duracion_segundos`     | histogram | metodo, ruta                      |
//! | `pokeapi_login_intentos_total`       | counter   | —                                 |
//! | `pokeapi_login_errores_total`        | counter   | motivo (usuario_no_existe/…)      |
//! | `pokeapi_usuarios_registrados_total` | counter   | —                                 |
//! | `pokeapi_cambios_rol_total`          | counter   | —                                 |
//! | `pokeapi_pokemon_consultas_total`    | counter   | origen (cache/api), resultado     |
//! | `pokeapi_upstream_peticiones_total`  | counter   | estado (HTTP o error_red)         |
//! | `pokeapi_upstream_duracion_segundos` | histogram | —                                 |
//! | `pokeapi_redis_operaciones_total`    | counter   | operacion, resultado              |
//! | `pokeapi_mongo_operaciones_total`    | counter   | operacion, resultado              |
//! | `pokeapi_sesiones_activas`           | gauge     | —                                 |
//! | `pokeapi_usuarios_por_rol`           | gauge     | rol                               |
//! | `pokeapi_redis_disponible`           | gauge     | —                                 |
//! | `pokeapi_mongodb_disponible`         | gauge     | —                                 |
//!
//! Login: cada intento suma en `pokeapi_login_intentos_total`; los fallos
//! suman además en `pokeapi_login_errores_total{motivo}`. Los aciertos =
//! `intentos - sum(errores)`. El motivo (usuario inexistente vs. password
//! incorrecto) se distingue aquí sin filtrarlo nunca al cliente.

use axum::extract::{MatchedPath, Request, State};
use axum::middleware::Next;
use axum::response::Response;
use prometheus::{
    Encoder, HistogramOpts, HistogramVec, IntCounter, IntCounterVec, IntGauge, IntGaugeVec,
    Opts, Registry, TextEncoder,
};

use crate::http::EstadoServidor;
use crate::sesion::SesionActual;

pub struct Metricas {
    registro: Registry,
    pub http_peticiones: IntCounterVec,
    pub http_duracion: HistogramVec,
    pub login_intentos: IntCounter,
    pub login_errores: IntCounterVec,
    pub usuarios_registrados: IntCounter,
    pub cambios_rol: IntCounter,
    pub pokemon_consultas: IntCounterVec,
    pub upstream_peticiones: IntCounterVec,
    pub upstream_duracion: prometheus::Histogram,
    pub redis_operaciones: IntCounterVec,
    pub mongo_operaciones: IntCounterVec,
    pub sesiones_activas: IntGauge,
    pub usuarios_por_rol: IntGaugeVec,
    pub redis_disponible: IntGauge,
    pub mongodb_disponible: IntGauge,
}

impl Metricas {
    /// # Errors
    ///
    /// Falla solo si una familia no puede registrarse (nombre duplicado), lo
    /// que sería un bug de programación detectado al arrancar.
    pub fn nuevas() -> Result<Self, prometheus::Error> {
        let registro = Registry::new();

        let http_peticiones = IntCounterVec::new(
            Opts::new("pokeapi_http_peticiones_total", "Peticiones HTTP atendidas"),
            &["metodo", "ruta", "estado", "rol"],
        )?;
        let http_duracion = HistogramVec::new(
            HistogramOpts::new(
                "pokeapi_http_duracion_segundos",
                "Duración de las peticiones HTTP",
            )
            .buckets(vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]),
            &["metodo", "ruta"],
        )?;
        let login_intentos = IntCounter::new(
            "pokeapi_login_intentos_total",
            "Intentos de inicio de sesión (todos, con éxito o no)",
        )?;
        let login_errores = IntCounterVec::new(
            Opts::new(
                "pokeapi_login_errores_total",
                "Inicios de sesión fallidos, por motivo",
            ),
            &["motivo"],
        )?;
        let usuarios_registrados = IntCounter::new(
            "pokeapi_usuarios_registrados_total",
            "Usuarios registrados desde el arranque",
        )?;
        let cambios_rol =
            IntCounter::new("pokeapi_cambios_rol_total", "Cambios de rol aplicados")?;
        let pokemon_consultas = IntCounterVec::new(
            Opts::new("pokeapi_pokemon_consultas_total", "Consultas de pokémon resueltas"),
            &["origen", "resultado"],
        )?;
        let upstream_peticiones = IntCounterVec::new(
            Opts::new(
                "pokeapi_upstream_peticiones_total",
                "Llamadas a la PokeAPI pública",
            ),
            &["estado"],
        )?;
        let upstream_duracion = prometheus::Histogram::with_opts(
            HistogramOpts::new(
                "pokeapi_upstream_duracion_segundos",
                "Duración de las llamadas a la PokeAPI",
            )
            .buckets(vec![0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
        )?;
        let redis_operaciones = IntCounterVec::new(
            Opts::new("pokeapi_redis_operaciones_total", "Operaciones contra Redis"),
            &["operacion", "resultado"],
        )?;
        let mongo_operaciones = IntCounterVec::new(
            Opts::new("pokeapi_mongo_operaciones_total", "Operaciones contra MongoDB"),
            &["operacion", "resultado"],
        )?;
        let sesiones_activas =
            IntGauge::new("pokeapi_sesiones_activas", "Sesiones vivas en Redis")?;
        let usuarios_por_rol = IntGaugeVec::new(
            Opts::new("pokeapi_usuarios_por_rol", "Usuarios existentes por rol"),
            &["rol"],
        )?;
        let redis_disponible = IntGauge::new(
            "pokeapi_redis_disponible",
            "1 si el último PING a Redis respondió, 0 si no",
        )?;
        let mongodb_disponible = IntGauge::new(
            "pokeapi_mongodb_disponible",
            "1 si el último ping a MongoDB respondió, 0 si no",
        )?;

        registro.register(Box::new(http_peticiones.clone()))?;
        registro.register(Box::new(http_duracion.clone()))?;
        registro.register(Box::new(login_intentos.clone()))?;
        registro.register(Box::new(login_errores.clone()))?;
        registro.register(Box::new(usuarios_registrados.clone()))?;
        registro.register(Box::new(cambios_rol.clone()))?;
        registro.register(Box::new(pokemon_consultas.clone()))?;
        registro.register(Box::new(upstream_peticiones.clone()))?;
        registro.register(Box::new(upstream_duracion.clone()))?;
        registro.register(Box::new(redis_operaciones.clone()))?;
        registro.register(Box::new(mongo_operaciones.clone()))?;
        registro.register(Box::new(sesiones_activas.clone()))?;
        registro.register(Box::new(usuarios_por_rol.clone()))?;
        registro.register(Box::new(redis_disponible.clone()))?;
        registro.register(Box::new(mongodb_disponible.clone()))?;

        Ok(Self {
            registro,
            http_peticiones,
            http_duracion,
            login_intentos,
            login_errores,
            usuarios_registrados,
            cambios_rol,
            pokemon_consultas,
            upstream_peticiones,
            upstream_duracion,
            redis_operaciones,
            mongo_operaciones,
            sesiones_activas,
            usuarios_por_rol,
            redis_disponible,
            mongodb_disponible,
        })
    }

    /// Exposición en el formato de texto de Prometheus (lo que rasca el
    /// servidor cada `scrape_interval`).
    ///
    /// # Errors
    ///
    /// Falla si el encoder no puede serializar alguna familia.
    pub fn texto(&self) -> Result<String, prometheus::Error> {
        let familias = self.registro.gather();
        let mut cuerpo = Vec::new();
        TextEncoder::new().encode(&familias, &mut cuerpo)?;
        Ok(String::from_utf8_lossy(&cuerpo).into_owned())
    }
}

/// Capa de observabilidad: resuelve la sesión (una vez por petición, queda en
/// las extensions para handlers y server functions) y registra contador +
/// histograma de cada respuesta.
pub async fn capa_http(
    State(estado): State<EstadoServidor>,
    mut peticion: Request,
    siguiente: Next,
) -> Response {
    let metodo = peticion.method().as_str().to_string();
    let ruta_cruda = peticion.uri().path().to_string();
    let es_estatico = ruta_cruda.starts_with("/pkg/") || ruta_cruda == "/favicon.ico";

    // Etiqueta `ruta` de baja cardinalidad: el patrón de la ruta si lo hay,
    // la ruta real para las server functions (nombres estables) y un cajón
    // común para los estáticos.
    let ruta = match peticion.extensions().get::<MatchedPath>() {
        Some(patron) if patron.as_str() == "/api/{*fn_name}" => ruta_cruda.clone(),
        Some(patron) => patron.as_str().to_string(),
        None if es_estatico => "/pkg/*".to_string(),
        None => "(otras)".to_string(),
    };

    let sesion = if es_estatico {
        SesionActual(None)
    } else {
        crate::sesion::resolver(&estado.contenedor, peticion.headers()).await
    };
    let rol = sesion.etiqueta_rol();
    peticion.extensions_mut().insert(sesion);

    let inicio = std::time::Instant::now();
    let respuesta = siguiente.run(peticion).await;
    let segundos = inicio.elapsed().as_secs_f64();

    let metricas = &estado.contenedor.metricas;
    metricas
        .http_peticiones
        .with_label_values(&[&metodo, &ruta, respuesta.status().as_str(), &rol])
        .inc();
    metricas.http_duracion.with_label_values(&[&metodo, &ruta]).observe(segundos);

    respuesta
}
