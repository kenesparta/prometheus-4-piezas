# pokeapi — la demo interactiva de "Prometheus en 4 piezas"

Aplicación **Rust + Leptos (SSR + hidratación)** con **MongoDB** (usuarios) y
**Redis** (sesiones y caché) como bases de datos, pensada para que el público
de la charla la use en vivo mientras las métricas aparecen en Prometheus.
Workspace organizado en estilo DDD/hexagonal (scaffold del skill `rs-proyecto`).

## Qué hace

- **Login** (`admin` / `123`) y **registro** (solo usuario + password; toda
  cuenta nueva entra con rol `VISITOR`). Cada intento se cuenta en Prometheus,
  y los fallos se desglosan por motivo (usuario inexistente vs. password
  incorrecto) sin revelárselo nunca a quien intenta entrar.
- **Roles** `ADMIN` / `EDITOR` / `VISITOR`; los usuarios viven en **MongoDB**:
  el panel `/admin` (solo ADMIN) promueve o degrada usuarios; `EDITOR` puede
  limpiar el historial.
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
| 4 · Alertmanager | Reglas `PokeapiCaida`, `PokeapiSinRedis`, `PokeapiSinMongo`, `PokeapiTraficoAltoDemo`, `PokeapiFuerzaBrutaDemo` |

## Estructura

- `crates/shared-kernel/` — errores de dominio compartidos.
- `crates/bc-identidad/` — usuarios, credenciales, roles y sesiones (solo
  `dominio/` + `aplicacion/`; sin IO).
- `crates/bc-pokedex/` — consultas de pokémon, caché e historial (ídem).
- `apps/backend/` — el binario: UI Leptos (`app/`), adaptadores HTTP
  (`http/`), persistencia MongoDB + Redis (`persistencia/`), cliente PokeAPI
  (`clientes/`), publicadores de eventos → logs + métricas (`mensajeria/`),
  métricas (`metricas.rs`) y wiring (`composicion.rs`).

Los BCs no dependen de tokio/axum/redis/mongodb: los adaptadores del binario
implementan sus puertos. La UI wasm tampoco arrastra el dominio: habla por
server functions con view-models propios.

## Correr en local

> ¿Solo quieres correr la **imagen ya publicada** (sin compilar)? Mira
> [`USO-IMAGEN.md`](USO-IMAGEN.md).

Requisitos: toolchain fijado por `rust-toolchain.toml` (1.96 + target wasm),
`cargo-leptos`, un Redis y un MongoDB accesibles.

```bash
# 1. Redis y MongoDB locales (en contenedores efímeros)
docker run --rm -p 6379:6379 redis:7-alpine
docker run --rm -p 27017:27017 mongo:7

# 2. Variables (o copia .env.ejemplo a .env y expórtalas)
export REDIS_URL=redis://127.0.0.1:6379
export MONGODB_URI=mongodb://127.0.0.1:27017

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
| `REDIS_URL` | — (obligatoria) | `redis://…` o `rediss://…` (TLS); sesiones y caché |
| `MONGODB_URI` | — (obligatoria) | `mongodb://…` o `mongodb+srv://…` (Atlas); usuarios |
| `MONGODB_DB` | `pokeapi` | Nombre de la base de datos en Mongo |
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
| `pokeapi_login_intentos_total` | counter | — |
| `pokeapi_login_errores_total` | counter | `motivo` (`usuario_no_existe`/`password_incorrecto`) |
| `pokeapi_usuarios_registrados_total` | counter | — |
| `pokeapi_cambios_rol_total` | counter | — |
| `pokeapi_pokemon_consultas_total` | counter | `origen` (`cache`/`api`), `resultado` |
| `pokeapi_upstream_peticiones_total` | counter | `estado` |
| `pokeapi_upstream_duracion_segundos` | histogram | — |
| `pokeapi_redis_operaciones_total` | counter | `operacion`, `resultado` |
| `pokeapi_mongo_operaciones_total` | counter | `operacion`, `resultado` |
| `pokeapi_sesiones_activas` | gauge | — |
| `pokeapi_usuarios_por_rol` | gauge | `rol` |
| `pokeapi_redis_disponible` | gauge | — |
| `pokeapi_mongodb_disponible` | gauge | — |

Queries PromQL bonitas para proyectar:

```promql
sum(rate(pokeapi_http_peticiones_total[1m])) by (rol)
sum(rate(pokeapi_pokemon_consultas_total[5m])) by (origen)
pokeapi_sesiones_activas
histogram_quantile(0.95, sum(rate(pokeapi_http_duracion_segundos_bucket[5m])) by (le))

# Login / seguridad: intentos, fallos por motivo y "fuerza bruta"
sum(rate(pokeapi_login_intentos_total[1m]))
sum(rate(pokeapi_login_errores_total[5m])) by (motivo)
sum(increase(pokeapi_login_errores_total{motivo="password_incorrecto"}[1m]))
```

## Desplegar

> Guía paso a paso para **usar la imagen** (local y Kubernetes):
> [`USO-IMAGEN.md`](USO-IMAGEN.md).

La imagen la construye y publica en **GitHub Packages (GHCR)** el CI/CD del
repositorio (workflow `.github/workflows/pokeapi-imagen.yml`) a partir de
`pokeapi/Dockerfile`, como `ghcr.io/kenesparta/pokeapi:latest` (+ `:<sha12>`).
Usa el `GITHUB_TOKEN` del propio workflow: no hay secrets que configurar.
Tras el **primer** push hay que hacer el package **público** (página del
package → Package settings → Change visibility) para que el cluster lo baje
sin `imagePullSecret`; `k8s/60-pokeapi.yaml` ya apunta a esa imagen.
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
`PokeapiFuerzaBrutaDemo` hace lo mismo con los fallos de login: tecleando mal
la contraseña unas cuantas veces (>5/min) pasa a Firing en vivo.
