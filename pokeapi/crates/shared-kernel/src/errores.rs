use thiserror::Error;

/// Errores de dominio compartidos entre Bounded Contexts.
///
/// Solo va aquí lo que de verdad significa lo mismo en más de un BC.
/// Si un error es específico de un contexto, defínelo en el `dominio/errores.rs`
/// de ese BC.
#[derive(Debug, Error)]
pub enum ErrorDominio {
    #[error("invariante violada: {0}")]
    Invariante(String),

    #[error("entidad no encontrada: {0}")]
    NoEncontrada(String),

    #[error("operación no permitida en el estado actual: {0}")]
    EstadoInvalido(String),
}
