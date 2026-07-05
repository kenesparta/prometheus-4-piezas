# Build multi-stage de la demo pokeapi (Leptos SSR + wasm).
#
# En CI la construye y publica .github/workflows/pokeapi-imagen.yml
# (GitHub Packages: ghcr.io/kenesparta/pokeapi). A mano, si hace falta:
#   docker buildx build --platform linux/amd64 \
#     -t ghcr.io/kenesparta/pokeapi:dev pokeapi/

# ---------- Etapa 1: compilación ----------
FROM rust:1.96-bookworm AS builder

# Target wasm para la hidratación + cargo-leptos (binario precompilado vía
# cargo-binstall: instalar desde código tarda ~10 min). Desde 0.3.x,
# cargo-leptos descarga el wasm-bindgen CLI que coincide con el Cargo.lock.
RUN rustup target add wasm32-unknown-unknown \
 && curl -L --proto '=https' --tlsv1.2 -sSf \
      https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
 && cargo binstall cargo-leptos@0.3.7 -y

WORKDIR /obra
COPY . .

RUN cd apps/backend && cargo leptos build --release

# ---------- Etapa 2: runtime ----------
FROM debian:bookworm-slim

# ca-certificates: TLS de salida hacia pokeapi.co y Redis en la nube.
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --uid 10001 pokeapi

WORKDIR /app
COPY --from=builder /obra/target/release/backend /app/backend
COPY --from=builder /obra/target/site /app/site

# Configuración de Leptos en producción (get_configuration la lee del entorno).
ENV LEPTOS_OUTPUT_NAME=pokeapi \
    LEPTOS_SITE_ROOT=/app/site \
    LEPTOS_SITE_PKG_DIR=pkg \
    LEPTOS_SITE_ADDR=0.0.0.0:3000 \
    LEPTOS_ENV=PROD \
    RUST_LOG=info

USER pokeapi
EXPOSE 3000
CMD ["/app/backend"]
