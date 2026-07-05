# pokeapi — la demo interactiva de "Prometheus en 4 piezas"

Aplicación **Rust + Leptos (SSR + hidratación)** con **Redis** como base de
datos, pensada para que el público de la charla la use en vivo mientras las
métricas aparecen en Prometheus. Workspace organizado en estilo DDD/hexagonal
(scaffold del skill `rs-proyecto`).

## Qué hace

- **Login** (`admin` / `123`) y **registro** (solo usuario + password; toda
  cuenta nueva entra con rol `VISITOR`).
- **Roles** `ADMIN` / `EDITOR` / `VISITOR` guardados en Redis: el panel
  `/admin` (solo ADMIN) promueve o degrada usuarios; `EDITOR` puede limpiar el
  historial.
- **Dashboard pokédex**: busca un pokémon; la primera consulta va a la
  PokeAPI pública (🌐) y las siguientes salen del **caché en Redis** (⚡) —
  la diferencia de latencia se muestra en pantalla y en las métricas.
- Cada consulta queda en una **bitácora en Redis** (visible en el dashboard) y
  todo el tráfico queda **instrumentado para Prometheus** en `/metrics`.

## Las 4 piezas, aplicadas a esta app

| Pieza | En esta app |
|---|---|
| 1 · Exporter | La propia app expone `/metrics` (formato texto, crate `prometheus`) |
| 2 · Servidor | El Prometheus del demo la scrapea como target `pokeapi:3000` (job `pokeapi`) |
| 3 · PromQL | `sum(rate(pokeapi_http_peticiones_total[1m])) by (rol)` y compañía |
| 4 · Alertmanager | Reglas `PokeapiCaida`, `PokeapiSinRedis`, `PokeapiTraficoAltoDemo` |

## Estructura

- `crates/shared-kernel/` — errores de dominio compartidos.
- `crates/bc-identidad/` — usuarios, credenciales, roles y sesiones (solo
  `dominio/` + `aplicacion/`; sin IO).
- `crates/bc-pokedex/` — consultas de pokémon, caché e historial (ídem).
- `apps/backend/` — el binario: UI Leptos (`app/`), adaptadores HTTP
  (`http/`), persistencia Redis (`persistencia/`), cliente PokeAPI
  (`clientes/`), publicadores de eventos → logs + métricas (`mensajeria/`),
  métricas (`metricas.rs`) y wiring (`composicion.rs`).

Los BCs no dependen de tokio/axum/redis: los adaptadores del binario
implementan sus puertos. La UI wasm tampoco arrastra el dominio: habla por
server functions con view-models propios.

## Correr en local

Requisitos: toolchain fijado por `rust-toolchain.toml` (1.96 + target wasm),
`cargo-leptos` y un Redis accesible.

```bash
# 1. Redis local (cualquiera de los dos)
docker run --rm -p 6379:6379 redis:7-alpine
redis-server --port 6379

# 2. Variables (o copia .env.ejemplo a .env y expórtalas)
export REDIS_URL=redis://127.0.0.1:6379

# 3. Levantar con recompilación en caliente
cd apps/backend && cargo leptos watch
# → http://127.0.0.1:3000  (admin / 123)
```

Verificaciones rápidas:

```bash
cargo check --workspace
cargo test  --workspace
cargo clippy --workspace
cargo check -p backend --no-default-features --features hydrate \
  --target wasm32-unknown-unknown
```

## Variables de entorno

| Variable | Default | Qué es |
|---|---|---|
| `REDIS_URL` | — (obligatoria) | `redis://…` o `rediss://…` (TLS) con credenciales |
| `ADMIN_PASSWORD` | `123` | Password del usuario `admin` sembrado al arrancar |
| `POKEAPI_URL_BASE` | `https://pokeapi.co/api/v2` | Base de la PokeAPI |
| `SESION_TTL_SEGUNDOS` | `86400` | TTL (deslizante) de las sesiones en Redis |
| `CACHE_TTL_SEGUNDOS` | `600` | TTL del caché de fichas en Redis |

## Endpoints para la demo por terminal

```bash
curl -s localhost:3000/api/pokemon/pikachu | jq '{nombre, origen}'
curl -s localhost:3000/api/historial | jq length
curl -s localhost:3000/metrics | grep '^pokeapi_'
curl -s localhost:3000/salud | jq

# Registrar y loguear por API (devuelve el token de sesión)
curl -s -X POST localhost:3000/api/registro \
  -H 'content-type: application/json' \
  -d '{"nombre":"ash","password":"paleta"}' | jq
curl -s -X POST localhost:3000/api/login \
  -H 'content-type: application/json' \
  -d '{"nombre":"ash","password":"paleta"}' | jq

# Generar tráfico anónimo para las métricas (rol="anonimo")
while true; do curl -s localhost:3000/api/pokemon/eevee >/dev/null; sleep 1; done
```

## Métricas expuestas (prefijo `pokeapi_`)

| Métrica | Tipo | Etiquetas |
|---|---|---|
| `pokeapi_http_peticiones_total` | counter | `metodo`, `ruta`, `estado`, `rol` |
| `pokeapi_http_duracion_segundos` | histogram | `metodo`, `ruta` |
| `pokeapi_logins_total` | counter | `resultado` (`exito`/`fallo`) |
| `pokeapi_usuarios_registrados_total` | counter | — |
| `pokeapi_cambios_rol_total` | counter | — |
| `pokeapi_pokemon_consultas_total` | counter | `origen` (`cache`/`api`), `resultado` |
| `pokeapi_upstream_peticiones_total` | counter | `estado` |
| `pokeapi_upstream_duracion_segundos` | histogram | — |
| `pokeapi_redis_operaciones_total` | counter | `operacion`, `resultado` |
| `pokeapi_sesiones_activas` | gauge | — |
| `pokeapi_usuarios_por_rol` | gauge | `rol` |
| `pokeapi_redis_disponible` | gauge | — |

Queries PromQL bonitas para proyectar:

```promql
sum(rate(pokeapi_http_peticiones_total[1m])) by (rol)
sum(rate(pokeapi_pokemon_consultas_total[5m])) by (origen)
pokeapi_sesiones_activas
histogram_quantile(0.95, sum(rate(pokeapi_http_duracion_segundos_bucket[5m])) by (le))
```

## Desplegar

La imagen la construye y publica en Docker Hub el **CI/CD del repositorio**
(workflow `.forgejo/workflows/pokeapi-imagen.yml`, Forgejo Actions) a partir
de `pokeapi/Dockerfile`; `k8s/60-pokeapi.yaml` debe apuntar a esa imagen.
Pasos manuales, si hacen falta:

```bash
# 1. Secrets con Redis/Mongo en la nube (credenciales reales en
#    k8s/secrets.local.yaml, gitignorado — nunca se commitean)
kubectl apply -f k8s/secrets.local.yaml

# 2. Desplegar app + scrape + alertas (ya integrados en k8s/)
kubectl apply -f k8s/60-pokeapi.yaml
kubectl apply -f k8s/20-prometheus-config.yaml   # y recargar Prometheus

# 3. URL pública para el público
kubectl -n prometheus-demo get svc pokeapi-publico \
  -o jsonpath='{.status.loadBalancer.ingress[0].ip}'
```

El target nuevo aparece en `http://localhost:9090/targets` (job `pokeapi`) y
las alertas en `/alerts`. `PokeapiTraficoAltoDemo` tiene umbral bajísimo a
propósito: se enciende en cuanto el público empieza a usar la app.
