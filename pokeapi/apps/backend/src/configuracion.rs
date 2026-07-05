use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Configuracion {
    /// URL de conexión a Redis (`redis://…` o `rediss://…` con TLS).
    pub redis_url: String,
    /// Password inicial del usuario `admin` (se siembra al arrancar).
    pub admin_password: String,
    /// Base de la PokeAPI pública.
    pub pokeapi_url_base: String,
    /// TTL de las sesiones (deslizante).
    pub sesion_ttl_segundos: u64,
    /// TTL del caché de fichas de pokémon en Redis.
    pub cache_ttl_segundos: u64,
}

impl Configuracion {
    pub fn desde_entorno() -> Result<Self> {
        Ok(Self {
            redis_url: std::env::var("REDIS_URL")
                .context("la variable REDIS_URL no está definida")?,
            admin_password: std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "123".to_string()),
            pokeapi_url_base: std::env::var("POKEAPI_URL_BASE")
                .unwrap_or_else(|_| "https://pokeapi.co/api/v2".to_string()),
            sesion_ttl_segundos: variable_numerica("SESION_TTL_SEGUNDOS", 86_400)?,
            cache_ttl_segundos: variable_numerica("CACHE_TTL_SEGUNDOS", 600)?,
        })
    }
}

fn variable_numerica(nombre: &str, por_defecto: u64) -> Result<u64> {
    match std::env::var(nombre) {
        Ok(valor) => valor
            .parse::<u64>()
            .with_context(|| format!("la variable {nombre} no es un número: {valor}")),
        Err(_) => Ok(por_defecto),
    }
}
