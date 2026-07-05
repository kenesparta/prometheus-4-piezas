//! Adaptadores de mensajería (publicación de Domain Events).
//!
//! Un archivo por Bounded Context. Cada publicador hace dos cosas con cada
//! evento: lo deja como log estructurado (`tracing`) y actualiza las
//! métricas Prometheus de negocio. Sustituir por una cola real (NATS, Kafka…)
//! solo tocaría estos archivos.

pub mod identidad_eventos;
pub mod pokedex_eventos;
