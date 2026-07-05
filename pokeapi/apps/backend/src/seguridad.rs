//! Adaptadores de criptografía y aleatoriedad para los puertos de identidad:
//! hasher de passwords (Argon2id) y generador de tokens de sesión (UUID v4).

use argon2::Argon2;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use rand_core::OsRng;
use bc_identidad::aplicacion::puertos::{ErrorHasher, GeneradorTokens, HasherPassword};
use bc_identidad::dominio::modelo::HashPassword;

#[derive(Debug, Default)]
pub struct HasherArgon2;

impl HasherPassword for HasherArgon2 {
    fn hashear(&self, password_plano: &str) -> Result<HashPassword, ErrorHasher> {
        let sal = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(password_plano.as_bytes(), &sal)
            .map_err(|e| ErrorHasher(e.to_string()))?;
        Ok(HashPassword::desde_cadena(hash.to_string()))
    }

    fn verificar(&self, password_plano: &str, hash: &HashPassword) -> bool {
        let Ok(parseado) = PasswordHash::new(hash.como_str()) else {
            return false;
        };
        Argon2::default()
            .verify_password(password_plano.as_bytes(), &parseado)
            .is_ok()
    }
}

#[derive(Debug, Default)]
pub struct GeneradorTokensUuid;

impl GeneradorTokens for GeneradorTokensUuid {
    fn generar(&self) -> String {
        uuid::Uuid::new_v4().simple().to_string()
    }
}
