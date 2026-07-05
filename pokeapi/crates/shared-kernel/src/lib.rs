//! Shared Kernel del workspace.
//!
//! Contiene únicamente tipos verdaderamente transversales que más de un
//! Bounded Context necesita compartir literalmente (no "por parecido").
//! Antes de añadir algo aquí, pregúntate si dos BCs realmente comparten
//! el mismo concepto o si conviene duplicarlo para evitar acoplamiento.

pub mod errores;

pub use errores::ErrorDominio;
