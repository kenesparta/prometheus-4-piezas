//! Repositorio MongoDB para el BC identidad: los **usuarios** de la app.
//!
//! Colección `usuarios`, un documento por usuario:
//! - `_id`           — el nombre de usuario (identidad natural del agregado).
//!   Al ser la clave primaria, insertar un `_id` repetido devuelve un error de
//!   clave duplicada: es el candado de unicidad atómico, sin índice extra.
//! - `hash_password` — hash Argon2id (opaco).
//! - `rol`           — `ADMIN` | `EDITOR` | `VISITOR`.
//! - `creado_en`     — RFC 3339 (mismo formato que usa el adaptador Redis).
//! - `version`       — contador optimista del agregado.

use std::sync::Arc;

use async_trait::async_trait;
use bc_identidad::dominio::modelo::{HashPassword, NombreUsuario, Rol, Usuario};
use bc_identidad::dominio::repositorio::{ErrorRepositorio, RepositorioUsuarios};
use chrono::{DateTime, Utc};
use futures_util::TryStreamExt;
use mongodb::bson::doc;
use mongodb::error::{ErrorKind, WriteFailure};
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};

use crate::metricas::Metricas;

const COLECCION_USUARIOS: &str = "usuarios";

/// Código de error de MongoDB para una violación de clave única.
const CODIGO_CLAVE_DUPLICADA: i32 = 11000;

/// Proyección BSON del agregado `Usuario`. `_id` = nombre para que Mongo
/// garantice la unicidad sin un índice adicional.
#[derive(Debug, Serialize, Deserialize)]
struct DocUsuario {
    #[serde(rename = "_id")]
    nombre: String,
    hash_password: String,
    rol: String,
    creado_en: String,
    version: i64,
}

impl DocUsuario {
    fn desde(usuario: &Usuario) -> Self {
        Self {
            nombre: usuario.nombre().como_str().to_string(),
            hash_password: usuario.hash_password().como_str().to_string(),
            rol: usuario.rol().como_str().to_string(),
            creado_en: usuario.creado_en().to_rfc3339(),
            version: i64::try_from(usuario.version()).unwrap_or(i64::MAX),
        }
    }

    fn hidratar(self) -> Result<Usuario, ErrorRepositorio> {
        let corrupto = |detalle: &str| {
            ErrorRepositorio::Infraestructura(format!("registro corrupto: {detalle}"))
        };

        let nombre = NombreUsuario::nuevo(self.nombre).map_err(|e| corrupto(&e.to_string()))?;
        let rol = Rol::desde_str(&self.rol).map_err(|e| corrupto(&e.to_string()))?;
        let creado_en = DateTime::parse_from_rfc3339(&self.creado_en)
            .map(|f| f.with_timezone(&Utc))
            .map_err(|_| corrupto("creado_en inválido"))?;
        let version = u64::try_from(self.version).unwrap_or(0);

        Ok(Usuario::hidratar(
            nombre,
            HashPassword::desde_cadena(self.hash_password),
            rol,
            creado_en,
            version,
        ))
    }
}

/// ¿El error corresponde a una clave `_id` duplicada?
fn es_clave_duplicada(error: &mongodb::error::Error) -> bool {
    matches!(
        &*error.kind,
        ErrorKind::Write(WriteFailure::WriteError(we)) if we.code == CODIGO_CLAVE_DUPLICADA
    )
}

pub struct RepositorioUsuariosMongo {
    coleccion: Collection<DocUsuario>,
    metricas: Arc<Metricas>,
}

impl RepositorioUsuariosMongo {
    pub fn nuevo(db: &Database, metricas: Arc<Metricas>) -> Self {
        Self {
            coleccion: db.collection::<DocUsuario>(COLECCION_USUARIOS),
            metricas,
        }
    }

    fn cuenta(&self, operacion: &'static str, ok: bool) {
        let resultado = if ok { "ok" } else { "error" };
        self.metricas
            .mongo_operaciones
            .with_label_values(&[operacion, resultado])
            .inc();
    }

    /// Contabiliza la operación en las métricas y traduce el error de Mongo al
    /// error del puerto.
    fn mapear<T>(
        &self,
        resultado: mongodb::error::Result<T>,
        operacion: &'static str,
    ) -> Result<T, ErrorRepositorio> {
        match resultado {
            Ok(valor) => {
                self.cuenta(operacion, true);
                Ok(valor)
            }
            Err(error) => {
                self.cuenta(operacion, false);
                Err(ErrorRepositorio::Infraestructura(error.to_string()))
            }
        }
    }
}

#[async_trait]
impl RepositorioUsuarios for RepositorioUsuariosMongo {
    async fn por_nombre(
        &self,
        nombre: &NombreUsuario,
    ) -> Result<Option<Usuario>, ErrorRepositorio> {
        let encontrado = self.mapear(
            self.coleccion.find_one(doc! { "_id": nombre.como_str() }).await,
            "find_one",
        )?;
        encontrado.map(DocUsuario::hidratar).transpose()
    }

    async fn guardar_nuevo(&self, usuario: &Usuario) -> Result<(), ErrorRepositorio> {
        match self.coleccion.insert_one(DocUsuario::desde(usuario)).await {
            Ok(_) => {
                self.cuenta("insert_one", true);
                Ok(())
            }
            // Clave duplicada no es un fallo de infraestructura: es el candado
            // de unicidad haciendo su trabajo. Cuenta como operación correcta.
            Err(error) if es_clave_duplicada(&error) => {
                self.cuenta("insert_one", true);
                Err(ErrorRepositorio::YaExiste)
            }
            Err(error) => {
                self.cuenta("insert_one", false);
                Err(ErrorRepositorio::Infraestructura(error.to_string()))
            }
        }
    }

    async fn guardar(&self, usuario: &Usuario) -> Result<(), ErrorRepositorio> {
        self.mapear(
            self.coleccion
                .replace_one(
                    doc! { "_id": usuario.nombre().como_str() },
                    DocUsuario::desde(usuario),
                )
                .await,
            "replace_one",
        )?;
        Ok(())
    }

    async fn listar(&self) -> Result<Vec<Usuario>, ErrorRepositorio> {
        let cursor = self.mapear(self.coleccion.find(doc! {}).await, "find")?;
        let documentos: Vec<DocUsuario> = self.mapear(cursor.try_collect().await, "cursor")?;

        let mut usuarios = Vec::with_capacity(documentos.len());
        for documento in documentos {
            match documento.hidratar() {
                Ok(usuario) => usuarios.push(usuario),
                Err(error) => tracing::warn!(%error, "usuario corrupto en Mongo; se omite"),
            }
        }
        Ok(usuarios)
    }
}
