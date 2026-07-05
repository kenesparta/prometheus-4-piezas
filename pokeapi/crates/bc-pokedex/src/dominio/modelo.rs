//! Modelo de dominio del BC pokedex.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_kernel::ErrorDominio;

// ============================================================================
// Value Object: NombrePokemon
// ============================================================================

/// Identificador de pokémon tal como lo espera la PokeAPI: minúsculas y
/// guiones (`pikachu`, `mr-mime`). Normaliza la entrada del usuario.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NombrePokemon(String);

impl NombrePokemon {
    pub fn nuevo(valor: impl Into<String>) -> Result<Self, ErrorDominio> {
        let v = valor.into().trim().to_lowercase().replace(' ', "-");
        if v.is_empty() {
            return Err(ErrorDominio::Invariante("escribe el nombre de un pokémon".into()));
        }
        if v.chars().count() > 50 {
            return Err(ErrorDominio::Invariante(
                "el nombre del pokémon supera los 50 caracteres".into(),
            ));
        }
        if !v.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return Err(ErrorDominio::Invariante(
                "el nombre del pokémon solo admite letras, números y '-'".into(),
            ));
        }
        Ok(Self(v))
    }

    pub fn como_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for NombrePokemon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// ============================================================================
// FichaPokemon
// ============================================================================

/// Proyección de un pokémon con lo que la aplicación necesita mostrar.
///
/// Es el modelo *nuestro*, no el de la PokeAPI: el adaptador (ACL) traduce la
/// respuesta externa a esta ficha, y es esta ficha la que se guarda en el
/// caché. Registro inmutable: se construye completo y no se muta.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FichaPokemon {
    pub numero: u32,
    pub nombre: String,
    pub tipos: Vec<String>,
    pub estadisticas: Vec<Estadistica>,
    /// Altura en decímetros, tal como la reporta la PokeAPI.
    pub altura_dm: u32,
    /// Peso en hectogramos, tal como lo reporta la PokeAPI.
    pub peso_hg: u32,
    pub sprite_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Estadistica {
    pub nombre: String,
    pub valor: u32,
}

// ============================================================================
// Consultas: origen y registro
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrigenConsulta {
    /// La ficha salió del caché en Redis.
    Cache,
    /// Hubo que ir a la PokeAPI pública.
    Api,
}

impl OrigenConsulta {
    pub fn como_str(&self) -> &'static str {
        match self {
            OrigenConsulta::Cache => "cache",
            OrigenConsulta::Api => "api",
        }
    }
}

/// Registro inmutable de una consulta hecha por alguien: quién, qué, de dónde
/// salió la respuesta y cuándo. Es lo que se guarda en la bitácora de Redis y
/// lo que el dashboard muestra en "consultas recientes".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsultaRegistrada {
    pub usuario: String,
    /// Rol de quien consultó como etiqueta plana (`ADMIN`, `EDITOR`,
    /// `VISITOR` o `anonimo`). Es metadato de la petición, no identidad:
    /// este BC no depende de bc-identidad.
    pub rol: String,
    pub pokemon: String,
    pub origen: OrigenConsulta,
    pub exito: bool,
    pub en: DateTime<Utc>,
}

impl ConsultaRegistrada {
    pub fn nueva(
        usuario: impl Into<String>,
        rol: impl Into<String>,
        pokemon: &NombrePokemon,
        origen: OrigenConsulta,
        exito: bool,
    ) -> Self {
        Self {
            usuario: usuario.into(),
            rol: rol.into(),
            pokemon: pokemon.como_str().to_string(),
            origen,
            exito,
            en: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nombre_pokemon_se_normaliza() {
        let nombre = NombrePokemon::nuevo("  Mr Mime ").expect("nombre válido");
        assert_eq!(nombre.como_str(), "mr-mime");
    }

    #[test]
    fn nombre_pokemon_rechaza_vacio_y_simbolos() {
        assert!(NombrePokemon::nuevo("   ").is_err());
        assert!(NombrePokemon::nuevo("pika!chu").is_err());
    }

    #[test]
    fn origen_como_etiqueta_de_metrica() {
        assert_eq!(OrigenConsulta::Cache.como_str(), "cache");
        assert_eq!(OrigenConsulta::Api.como_str(), "api");
    }

    #[test]
    fn consulta_registrada_copia_el_nombre_normalizado() {
        let nombre = NombrePokemon::nuevo("Pikachu").expect("nombre válido");
        let consulta =
            ConsultaRegistrada::nueva("ash", "VISITOR", &nombre, OrigenConsulta::Api, true);
        assert_eq!(consulta.pokemon, "pikachu");
        assert!(consulta.exito);
    }
}
