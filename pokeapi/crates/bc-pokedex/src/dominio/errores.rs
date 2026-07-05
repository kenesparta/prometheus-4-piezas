//! Errores propios del dominio del BC pokedex.
//!
//! Hoy el contexto no tiene errores de dominio específicos: las validaciones
//! usan `shared_kernel::ErrorDominio` y los fallos de infraestructura viven
//! en los puertos (`ErrorRepositorio`, `ErrorFuente`). El módulo existe para
//! mantener la forma del scaffold y acoger errores futuros.
