//! Capa de aplicación del BC identidad.
//!
//! Orquesta los casos de uso: carga el agregado vía repositorio, invoca
//! operaciones de dominio, persiste y publica eventos. No contiene reglas de
//! negocio (eso es del dominio) ni detalles de transporte (eso es del
//! adaptador HTTP/Leptos en el binario).

pub mod casos_uso;
pub mod dto;
pub mod puertos;
