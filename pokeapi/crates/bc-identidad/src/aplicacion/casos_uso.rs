//! Casos de uso del BC identidad.
//!
//! Patrón por caso de uso:
//!   1. Validar/convertir DTO a tipos de dominio (VOs).
//!   2. Cargar agregado vía repositorio (o crearlo).
//!   3. Invocar la operación de dominio (que aplica invariantes y emite eventos).
//!   4. Persistir.
//!   5. Drenar y publicar eventos.
//!
//! Los eventos del ciclo de vida de la sesión (iniciada, fallida, cerrada) no
//! nacen de un agregado mutado, así que los construye el propio caso de uso y
//! los entrega igual al publicador.

use std::sync::Arc;

use chrono::Utc;
use shared_kernel::ErrorDominio;
use thiserror::Error;

use super::dto::{
    CambiarRolCmd, IniciarSesionCmd, RegistrarUsuarioCmd, VistaSesion, VistaUsuario,
};
use super::puertos::{ErrorHasher, GeneradorTokens, HasherPassword, PublicadorEventos};
use crate::dominio::errores::ErrorIdentidad;
use crate::dominio::eventos::{EventoIdentidad, MotivoFallo};
use crate::dominio::modelo::{NombreUsuario, Rol, Sesion, Usuario, validar_password};
use crate::dominio::repositorio::{
    ErrorRepositorio, RepositorioSesiones, RepositorioUsuarios,
};

#[derive(Debug, Error)]
pub enum ErrorCasoUso {
    #[error(transparent)]
    Dominio(#[from] ErrorDominio),

    #[error(transparent)]
    Identidad(#[from] ErrorIdentidad),

    #[error(transparent)]
    Repositorio(#[from] ErrorRepositorio),

    #[error(transparent)]
    Hasher(#[from] ErrorHasher),

    #[error("ese nombre de usuario ya está tomado")]
    NombreTomado,

    #[error("usuario o password incorrectos")]
    CredencialesInvalidas,

    #[error("operación reservada al rol ADMIN")]
    NoAutorizado,

    #[error("usuario no encontrado")]
    NoEncontrado,
}

fn vista_usuario(usuario: &Usuario) -> VistaUsuario {
    VistaUsuario {
        nombre: usuario.nombre().como_str().to_string(),
        rol: usuario.rol().como_str().to_string(),
        creado_en: usuario.creado_en().to_rfc3339(),
    }
}

fn vista_sesion(sesion: &Sesion) -> VistaSesion {
    VistaSesion {
        token: sesion.token().to_string(),
        nombre_usuario: sesion.nombre_usuario().como_str().to_string(),
        rol: sesion.rol().como_str().to_string(),
    }
}

// ============================================================================
// RegistrarUsuario
// ============================================================================

pub struct RegistrarUsuario {
    usuarios: Arc<dyn RepositorioUsuarios>,
    hasher: Arc<dyn HasherPassword>,
    publicador: Arc<dyn PublicadorEventos>,
}

impl RegistrarUsuario {
    pub fn nuevo(
        usuarios: Arc<dyn RepositorioUsuarios>,
        hasher: Arc<dyn HasherPassword>,
        publicador: Arc<dyn PublicadorEventos>,
    ) -> Self {
        Self { usuarios, hasher, publicador }
    }

    /// # Errors
    ///
    /// - [`ErrorCasoUso::Dominio`] si el nombre o el password no cumplen las reglas.
    /// - [`ErrorCasoUso::NombreTomado`] si el nombre ya existe.
    pub async fn ejecutar(&self, cmd: RegistrarUsuarioCmd) -> Result<VistaUsuario, ErrorCasoUso> {
        let nombre = NombreUsuario::nuevo(cmd.nombre)?;
        validar_password(&cmd.password)?;
        let hash = self.hasher.hashear(&cmd.password)?;

        let mut usuario = Usuario::registrar(nombre, hash);
        match self.usuarios.guardar_nuevo(&usuario).await {
            Ok(()) => {}
            Err(ErrorRepositorio::YaExiste) => return Err(ErrorCasoUso::NombreTomado),
            Err(otro) => return Err(otro.into()),
        }

        let eventos = usuario.drenar_eventos();
        self.publicador.publicar(&eventos).await;
        Ok(vista_usuario(&usuario))
    }
}

// ============================================================================
// IniciarSesion
// ============================================================================

pub struct IniciarSesion {
    usuarios: Arc<dyn RepositorioUsuarios>,
    sesiones: Arc<dyn RepositorioSesiones>,
    hasher: Arc<dyn HasherPassword>,
    tokens: Arc<dyn GeneradorTokens>,
    publicador: Arc<dyn PublicadorEventos>,
    ttl_segundos: u64,
}

impl IniciarSesion {
    pub fn nuevo(
        usuarios: Arc<dyn RepositorioUsuarios>,
        sesiones: Arc<dyn RepositorioSesiones>,
        hasher: Arc<dyn HasherPassword>,
        tokens: Arc<dyn GeneradorTokens>,
        publicador: Arc<dyn PublicadorEventos>,
        ttl_segundos: u64,
    ) -> Self {
        Self { usuarios, sesiones, hasher, tokens, publicador, ttl_segundos }
    }

    /// # Errors
    ///
    /// Devuelve siempre [`ErrorCasoUso::CredencialesInvalidas`] ante usuario
    /// inexistente, nombre mal formado o password incorrecto: no se filtra
    /// cuál de los tres falló (evita enumerar usuarios).
    pub async fn ejecutar(&self, cmd: IniciarSesionCmd) -> Result<VistaSesion, ErrorCasoUso> {
        // Todo intento cuenta, gane o pierda: se emite antes de resolverlo.
        self.publicador
            .publicar(&[EventoIdentidad::IntentoLogin {
                nombre: cmd.nombre.trim().to_lowercase(),
                en: Utc::now(),
            }])
            .await;

        let usuario = match NombreUsuario::nuevo(cmd.nombre.clone()) {
            Ok(nombre) => self.usuarios.por_nombre(&nombre).await?,
            Err(_) => None,
        };

        let Some(usuario) = usuario else {
            self.publicar_fallo(&cmd.nombre, MotivoFallo::UsuarioNoExiste).await;
            return Err(ErrorCasoUso::CredencialesInvalidas);
        };

        if !self.hasher.verificar(&cmd.password, usuario.hash_password()) {
            self.publicar_fallo(&cmd.nombre, MotivoFallo::PasswordIncorrecto).await;
            return Err(ErrorCasoUso::CredencialesInvalidas);
        }

        let sesion = Sesion::nueva(
            self.tokens.generar(),
            usuario.nombre().clone(),
            usuario.rol(),
        )?;
        self.sesiones.guardar(&sesion, self.ttl_segundos).await?;

        self.publicador
            .publicar(&[EventoIdentidad::SesionIniciada {
                nombre: usuario.nombre().clone(),
                rol: usuario.rol(),
                en: Utc::now(),
            }])
            .await;

        Ok(vista_sesion(&sesion))
    }

    async fn publicar_fallo(&self, nombre: &str, motivo: MotivoFallo) {
        self.publicador
            .publicar(&[EventoIdentidad::LoginFallido {
                nombre: nombre.trim().to_lowercase(),
                motivo,
                en: Utc::now(),
            }])
            .await;
    }
}

// ============================================================================
// ValidarSesion
// ============================================================================

pub struct ValidarSesion {
    sesiones: Arc<dyn RepositorioSesiones>,
    ttl_segundos: u64,
}

impl ValidarSesion {
    pub fn nuevo(sesiones: Arc<dyn RepositorioSesiones>, ttl_segundos: u64) -> Self {
        Self { sesiones, ttl_segundos }
    }

    /// Devuelve la sesión asociada al token (renovando su TTL) o `None` si el
    /// token no existe o ya expiró.
    pub async fn ejecutar(&self, token: &str) -> Result<Option<VistaSesion>, ErrorCasoUso> {
        if token.trim().is_empty() {
            return Ok(None);
        }
        let sesion = self.sesiones.por_token(token, self.ttl_segundos).await?;
        Ok(sesion.as_ref().map(vista_sesion))
    }
}

// ============================================================================
// CerrarSesion
// ============================================================================

pub struct CerrarSesion {
    sesiones: Arc<dyn RepositorioSesiones>,
    publicador: Arc<dyn PublicadorEventos>,
}

impl CerrarSesion {
    pub fn nuevo(
        sesiones: Arc<dyn RepositorioSesiones>,
        publicador: Arc<dyn PublicadorEventos>,
    ) -> Self {
        Self { sesiones, publicador }
    }

    /// Idempotente: cerrar una sesión inexistente no es un error.
    pub async fn ejecutar(&self, token: &str) -> Result<(), ErrorCasoUso> {
        let sesion = self.sesiones.por_token(token, 1).await?;
        self.sesiones.eliminar(token).await?;

        if let Some(sesion) = sesion {
            self.publicador
                .publicar(&[EventoIdentidad::SesionCerrada {
                    nombre: sesion.nombre_usuario().clone(),
                    en: Utc::now(),
                }])
                .await;
        }
        Ok(())
    }
}

// ============================================================================
// ListarUsuarios (solo ADMIN)
// ============================================================================

pub struct ListarUsuarios {
    usuarios: Arc<dyn RepositorioUsuarios>,
}

impl ListarUsuarios {
    pub fn nuevo(usuarios: Arc<dyn RepositorioUsuarios>) -> Self {
        Self { usuarios }
    }

    /// # Errors
    ///
    /// [`ErrorCasoUso::NoAutorizado`] si quien pide la lista no es ADMIN.
    pub async fn ejecutar(&self, rol_solicitante: &str) -> Result<Vec<VistaUsuario>, ErrorCasoUso> {
        if !Rol::desde_str(rol_solicitante)?.es_admin() {
            return Err(ErrorCasoUso::NoAutorizado);
        }
        let mut usuarios = self.usuarios.listar().await?;
        usuarios.sort_by(|a, b| a.nombre().como_str().cmp(b.nombre().como_str()));
        Ok(usuarios.iter().map(vista_usuario).collect())
    }
}

// ============================================================================
// CambiarRolUsuario (solo ADMIN)
// ============================================================================

pub struct CambiarRolUsuario {
    usuarios: Arc<dyn RepositorioUsuarios>,
    publicador: Arc<dyn PublicadorEventos>,
}

impl CambiarRolUsuario {
    pub fn nuevo(
        usuarios: Arc<dyn RepositorioUsuarios>,
        publicador: Arc<dyn PublicadorEventos>,
    ) -> Self {
        Self { usuarios, publicador }
    }

    /// # Errors
    ///
    /// - [`ErrorCasoUso::NoAutorizado`] si el solicitante no es ADMIN.
    /// - [`ErrorCasoUso::NoEncontrado`] si el usuario objetivo no existe.
    /// - [`ErrorIdentidad::UltimoAdmin`] si degradaría al último ADMIN.
    pub async fn ejecutar(&self, cmd: CambiarRolCmd) -> Result<VistaUsuario, ErrorCasoUso> {
        if !Rol::desde_str(&cmd.rol_solicitante)?.es_admin() {
            return Err(ErrorCasoUso::NoAutorizado);
        }

        let nombre = NombreUsuario::nuevo(cmd.nombre_objetivo)?;
        let nuevo_rol = Rol::desde_str(&cmd.nuevo_rol)?;

        let mut usuario = self
            .usuarios
            .por_nombre(&nombre)
            .await?
            .ok_or(ErrorCasoUso::NoEncontrado)?;

        if usuario.rol().es_admin() && !nuevo_rol.es_admin() {
            let admins = self
                .usuarios
                .listar()
                .await?
                .iter()
                .filter(|u| u.rol().es_admin())
                .count();
            if admins <= 1 {
                return Err(ErrorIdentidad::UltimoAdmin.into());
            }
        }

        usuario.cambiar_rol(nuevo_rol);
        self.usuarios.guardar(&usuario).await?;

        let eventos = usuario.drenar_eventos();
        self.publicador.publicar(&eventos).await;
        Ok(vista_usuario(&usuario))
    }
}
