//! Modelo de dominio del BC identidad.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_kernel::ErrorDominio;

use super::eventos::EventoIdentidad;

// ============================================================================
// Value Objects
// ============================================================================

/// Nombre de usuario normalizado (minúsculas, sin espacios alrededor).
///
/// Es también la identidad natural del agregado `Usuario`: no hay dos
/// usuarios con el mismo nombre.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NombreUsuario(String);

impl NombreUsuario {
    pub fn nuevo(valor: impl Into<String>) -> Result<Self, ErrorDominio> {
        let v = valor.into().trim().to_lowercase();
        let largo = v.chars().count();
        if largo < 3 {
            return Err(ErrorDominio::Invariante(
                "el nombre de usuario necesita al menos 3 caracteres".into(),
            ));
        }
        if largo > 30 {
            return Err(ErrorDominio::Invariante(
                "el nombre de usuario supera los 30 caracteres".into(),
            ));
        }
        if !v.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')) {
            return Err(ErrorDominio::Invariante(
                "el nombre de usuario solo admite letras, números, '_', '-' y '.'".into(),
            ));
        }
        Ok(Self(v))
    }

    pub fn como_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for NombreUsuario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Rol de un usuario dentro de la aplicación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Rol {
    Admin,
    Editor,
    Visitor,
}

impl Rol {
    pub const TODOS: [Rol; 3] = [Rol::Admin, Rol::Editor, Rol::Visitor];

    pub fn como_str(&self) -> &'static str {
        match self {
            Rol::Admin => "ADMIN",
            Rol::Editor => "EDITOR",
            Rol::Visitor => "VISITOR",
        }
    }

    pub fn desde_str(valor: &str) -> Result<Self, ErrorDominio> {
        match valor.trim().to_uppercase().as_str() {
            "ADMIN" => Ok(Rol::Admin),
            "EDITOR" => Ok(Rol::Editor),
            "VISITOR" => Ok(Rol::Visitor),
            otro => Err(ErrorDominio::Invariante(format!("rol desconocido: {otro}"))),
        }
    }

    pub fn es_admin(&self) -> bool {
        matches!(self, Rol::Admin)
    }

    /// EDITOR y ADMIN pueden realizar acciones de edición (p. ej. limpiar el
    /// historial de consultas).
    pub fn puede_editar(&self) -> bool {
        matches!(self, Rol::Admin | Rol::Editor)
    }
}

impl std::fmt::Display for Rol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.como_str())
    }
}

/// Hash de password ya derivado (opaco para el dominio).
///
/// El dominio nunca ve el password en claro más allá de validarlo; derivar y
/// verificar el hash es trabajo del puerto `HasherPassword`.
#[derive(Clone)]
pub struct HashPassword(String);

impl HashPassword {
    pub fn desde_cadena(valor: impl Into<String>) -> Self {
        Self(valor.into())
    }

    pub fn como_str(&self) -> &str {
        &self.0
    }
}

// No derivamos `Debug`: el hash no debe terminar en logs por accidente.
impl std::fmt::Debug for HashPassword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("HashPassword(****)")
    }
}

/// Reglas mínimas para un password en claro, validadas antes de hashear.
pub fn validar_password(plano: &str) -> Result<(), ErrorDominio> {
    let largo = plano.chars().count();
    if largo < 3 {
        return Err(ErrorDominio::Invariante(
            "el password necesita al menos 3 caracteres".into(),
        ));
    }
    if largo > 72 {
        return Err(ErrorDominio::Invariante("el password supera los 72 caracteres".into()));
    }
    Ok(())
}

// ============================================================================
// Aggregate Root: Usuario
// ============================================================================
//
// Garantiza invariantes en cada operación y emite Domain Events. No conoce
// persistencia ni transporte: el repositorio lo materializa desde fuera.

#[derive(Debug, Clone)]
pub struct Usuario {
    nombre: NombreUsuario,
    hash_password: HashPassword,
    rol: Rol,
    creado_en: DateTime<Utc>,
    version: u64,
    eventos_pendientes: Vec<EventoIdentidad>,
}

impl Usuario {
    /// Registro normal desde la vista pública: siempre entra como VISITOR.
    pub fn registrar(nombre: NombreUsuario, hash_password: HashPassword) -> Self {
        Self::registrar_con_rol(nombre, hash_password, Rol::Visitor)
    }

    /// Registro con rol explícito. Lo usa la composición del binario para
    /// sembrar la cuenta `admin` inicial.
    pub fn registrar_con_rol(
        nombre: NombreUsuario,
        hash_password: HashPassword,
        rol: Rol,
    ) -> Self {
        let creado_en = Utc::now();
        let mut usuario = Self {
            nombre: nombre.clone(),
            hash_password,
            rol,
            creado_en,
            version: 0,
            eventos_pendientes: Vec::new(),
        };
        usuario.registrar_evento(EventoIdentidad::UsuarioRegistrado {
            nombre,
            rol,
            en: creado_en,
        });
        usuario
    }

    /// Constructor de hidratación: úsalo solo desde el repositorio al cargar
    /// el usuario desde Redis. No emite eventos.
    pub fn hidratar(
        nombre: NombreUsuario,
        hash_password: HashPassword,
        rol: Rol,
        creado_en: DateTime<Utc>,
        version: u64,
    ) -> Self {
        Self {
            nombre,
            hash_password,
            rol,
            creado_en,
            version,
            eventos_pendientes: Vec::new(),
        }
    }

    pub fn nombre(&self) -> &NombreUsuario {
        &self.nombre
    }

    pub fn hash_password(&self) -> &HashPassword {
        &self.hash_password
    }

    pub fn rol(&self) -> Rol {
        self.rol
    }

    pub fn creado_en(&self) -> DateTime<Utc> {
        self.creado_en
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    /// Cambia el rol del usuario. Es un no-op si el rol es el mismo.
    pub fn cambiar_rol(&mut self, nuevo: Rol) {
        if nuevo == self.rol {
            return;
        }
        let anterior = self.rol;
        self.rol = nuevo;
        self.version += 1;
        self.registrar_evento(EventoIdentidad::RolCambiado {
            nombre: self.nombre.clone(),
            anterior,
            nuevo,
            en: Utc::now(),
        });
    }

    fn registrar_evento(&mut self, evento: EventoIdentidad) {
        self.eventos_pendientes.push(evento);
    }

    /// Devuelve y vacía los eventos pendientes. El caso de uso lo invoca tras
    /// persistir el agregado para entregárselos al publicador.
    pub fn drenar_eventos(&mut self) -> Vec<EventoIdentidad> {
        std::mem::take(&mut self.eventos_pendientes)
    }
}

// ============================================================================
// Entidad: Sesion
// ============================================================================
//
// Vive en Redis con TTL; el token es su identidad. Guarda una foto del rol
// para etiquetar métricas sin recargar al usuario en cada petición (las
// operaciones sensibles vuelven a validar contra el agregado `Usuario`).

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sesion {
    token: String,
    nombre_usuario: NombreUsuario,
    rol: Rol,
    creada_en: DateTime<Utc>,
}

impl Sesion {
    pub fn nueva(
        token: impl Into<String>,
        nombre_usuario: NombreUsuario,
        rol: Rol,
    ) -> Result<Self, ErrorDominio> {
        let token = token.into();
        if token.trim().is_empty() {
            return Err(ErrorDominio::Invariante("el token de sesión no puede estar vacío".into()));
        }
        Ok(Self {
            token,
            nombre_usuario,
            rol,
            creada_en: Utc::now(),
        })
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn nombre_usuario(&self) -> &NombreUsuario {
        &self.nombre_usuario
    }

    pub fn rol(&self) -> Rol {
        self.rol
    }

    pub fn creada_en(&self) -> DateTime<Utc> {
        self.creada_en
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nombre_usuario_se_normaliza_a_minusculas() {
        let nombre = NombreUsuario::nuevo("  PikaFan99 ").expect("nombre válido");
        assert_eq!(nombre.como_str(), "pikafan99");
    }

    #[test]
    fn nombre_usuario_rechaza_cortos_y_caracteres_invalidos() {
        assert!(NombreUsuario::nuevo("ab").is_err());
        assert!(NombreUsuario::nuevo("hola mundo").is_err());
        assert!(NombreUsuario::nuevo("ash@ketchum").is_err());
    }

    #[test]
    fn rol_ida_y_vuelta_desde_cadena() {
        for rol in Rol::TODOS {
            assert_eq!(Rol::desde_str(rol.como_str()).expect("rol válido"), rol);
        }
        assert!(Rol::desde_str("SUPERUSUARIO").is_err());
    }

    #[test]
    fn registrar_asigna_visitor_y_emite_evento() {
        let nombre = NombreUsuario::nuevo("misty").expect("nombre válido");
        let mut usuario =
            Usuario::registrar(nombre, HashPassword::desde_cadena("$argon2id$fake"));
        assert_eq!(usuario.rol(), Rol::Visitor);
        let eventos = usuario.drenar_eventos();
        assert_eq!(eventos.len(), 1);
        assert!(matches!(eventos[0], EventoIdentidad::UsuarioRegistrado { .. }));
        assert!(usuario.drenar_eventos().is_empty(), "drenar debe vaciar la cola");
    }

    #[test]
    fn cambiar_rol_al_mismo_valor_no_emite_evento() {
        let nombre = NombreUsuario::nuevo("brock").expect("nombre válido");
        let mut usuario =
            Usuario::registrar(nombre, HashPassword::desde_cadena("$argon2id$fake"));
        usuario.drenar_eventos();

        usuario.cambiar_rol(Rol::Visitor);
        assert!(usuario.drenar_eventos().is_empty());
        assert_eq!(usuario.version(), 0);

        usuario.cambiar_rol(Rol::Editor);
        let eventos = usuario.drenar_eventos();
        assert_eq!(eventos.len(), 1);
        assert_eq!(usuario.version(), 1);
        assert!(matches!(
            eventos[0],
            EventoIdentidad::RolCambiado { nuevo: Rol::Editor, .. }
        ));
    }

    #[test]
    fn sesion_rechaza_token_vacio() {
        let nombre = NombreUsuario::nuevo("serena").expect("nombre válido");
        assert!(Sesion::nueva("   ", nombre, Rol::Visitor).is_err());
    }

    #[test]
    fn password_valida_largo_minimo() {
        assert!(validar_password("12").is_err());
        assert!(validar_password("123").is_ok());
    }
}
