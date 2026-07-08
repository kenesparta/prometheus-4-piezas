# Usar la imagen Docker de pokeapi (local y en Kubernetes)

Cómo correr la **imagen ya publicada** `ghcr.io/kenesparta/pokeapi` sin compilar
nada. Si lo que quieres es desarrollar o compilar desde el código fuente, mira
[`README.md`](README.md).

## La imagen

| | |
|---|---|
| Registro | GitHub Packages (GHCR), **público** |
| Nombre | `ghcr.io/kenesparta/pokeapi` |
| Tags | `latest` (último `main`) y `<sha12>` (commit, inmutable) |
| Arquitectura | `linux/amd64` (en Apple Silicon corre por emulación) |
| Puerto | `3000` (HTTP, escucha en `0.0.0.0:3000`) |
| Endpoints | `/` UI · `/vivo` liveness · `/salud` estado deps · `/metrics` Prometheus |
| La construye | `.github/workflows/pokeapi-imagen.yml` desde `pokeapi/Dockerfile` |

La config de Leptos va **horneada** en la imagen (`LEPTOS_SITE_ADDR=0.0.0.0:3000`,
etc.): no hace falta pasar esas variables. Lo **obligatorio** es `REDIS_URL`
(sesiones y caché) y `MONGODB_URI` (usuarios), apuntando a un Redis y un MongoDB
accesibles; el resto tiene default (ver [Variables](#variables)).

```bash
docker pull ghcr.io/kenesparta/pokeapi:latest
```

> El package es público: se baja sin login ni `imagePullSecret`. Para fijar una
> versión reproducible usa el tag de commit (`:<sha12>`) en vez de `:latest`.

## En local (Docker)

La app necesita un Redis y un MongoDB. Lo más simple es levantar los tres juntos.

### Opción A — Docker Compose (recomendada)

Guarda esto como `compose.yaml` y `docker compose up`:

```yaml
services:
  redis:
    image: redis:7-alpine
    # Publica 6379 solo si quieres inspeccionar con redis-cli desde el host.
    ports: ["6379:6379"]

  mongo:
    image: mongo:7
    # Publica 27017 solo si quieres inspeccionar con mongosh desde el host.
    ports: ["27017:27017"]

  pokeapi:
    image: ghcr.io/kenesparta/pokeapi:latest
    # platform: linux/amd64   # descoméntalo en Apple Silicon para silenciar el aviso
    ports: ["3000:3000"]
    environment:
      REDIS_URL: redis://redis:6379     # 'redis' = nombre del servicio de arriba
      MONGODB_URI: mongodb://mongo:27017 # 'mongo' = nombre del servicio de arriba
      ADMIN_PASSWORD: "123"
      RUST_LOG: "info,backend=info"
    depends_on: [redis, mongo]
```

→ http://localhost:3000 (entra con `admin` / `123`).

### Opción B — `docker run`

Redis, MongoDB y pokeapi en una red compartida, para que se vean por nombre:

```bash
docker network create pokeapi-net

docker run -d --name redis --network pokeapi-net redis:7-alpine
docker run -d --name mongo --network pokeapi-net mongo:7

docker run --rm --network pokeapi-net -p 3000:3000 \
  -e REDIS_URL=redis://redis:6379 \
  -e MONGODB_URI=mongodb://mongo:27017 \
  -e ADMIN_PASSWORD=123 \
  ghcr.io/kenesparta/pokeapi:latest

# Limpieza al terminar:
#   docker rm -f redis mongo && docker network rm pokeapi-net
```

> ¿Ya tienes un Redis o un MongoDB (local o en la nube)? Sáltate ese contenedor
> y apunta la variable a tu instancia:
> - En el **host** (Docker Desktop): `redis://host.docker.internal:6379`,
>   `mongodb://host.docker.internal:27017`
> - En la **nube** con TLS: `rediss://usuario:PASSWORD@host:puerto`,
>   `mongodb+srv://usuario:PASSWORD@host/...`

### Comprobar

```bash
curl -s localhost:3000/salud                    # healthcheck (el mismo que usan las probes)
curl -s localhost:3000/metrics | grep '^pokeapi_'
curl -s localhost:3000/api/pokemon/pikachu | jq '{nombre, origen}'
```

(Los endpoints completos de la demo están en
[`README.md`](README.md#endpoints-para-la-demo-por-terminal).)

## En Kubernetes

### A — En este repo (la demo de la charla)

La imagen ya está cableada en [`k8s/60-pokeapi.yaml`](../k8s/60-pokeapi.yaml):
Deployment + Service interno (`pokeapi:3000`, el target que scrapea Prometheus) +
Service público (`pokeapi-publico`, LoadBalancer). `REDIS_URL` sale del Secret
`pokeapi-redis` y `MONGODB_URI` del Secret `pokeapi-mongodb`.

```bash
# 0. (una vez) el package de GHCR debe ser PÚBLICO; si no → ImagePullBackOff.

# 1. Namespace + Secrets con las URLs de Redis y MongoDB.
kubectl apply -f k8s/00-namespace.yaml
kubectl apply -f k8s/secrets.local.yaml      # gitignorado; credenciales reales

#    ...o crea los Secrets a mano (placeholder — pon tus URLs reales):
kubectl create secret generic pokeapi-redis \
  --namespace prometheus-demo \
  --from-literal=REDIS_URL='rediss://usuario:PASSWORD@host:puerto'
kubectl create secret generic pokeapi-mongodb \
  --namespace prometheus-demo \
  --from-literal=MONGODB_URI='mongodb+srv://usuario:PASSWORD@host/...'

# 2. Desplegar la app.
kubectl apply -f k8s/60-pokeapi.yaml
kubectl -n prometheus-demo rollout status deploy/pokeapi

# 3. Probar sin exponerla: port-forward del Service interno.
kubectl -n prometheus-demo port-forward svc/pokeapi 3000:3000
# → http://localhost:3000

# 4. URL pública (si usas el LoadBalancer):
kubectl -n prometheus-demo get svc pokeapi-publico \
  -o jsonpath='{.status.loadBalancer.ingress[0].ip}'
```

> En este repo el despliegue lo hace normalmente el CI
> (`.github/workflows/desplegar-k8s.yml`, en cada push que toque `k8s/**`). Lo de
> arriba es el equivalente manual.

### B — En cualquier cluster (manifiesto mínimo)

Si solo quieres correr la imagen en tu propio cluster, sin el resto de la demo,
esto basta (guárdalo como `pokeapi.yaml`):

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: pokeapi-redis
type: Opaque
stringData:
  REDIS_URL: rediss://usuario:PASSWORD@host:puerto   # pon TU URL real (no la commitees)
---
apiVersion: v1
kind: Secret
metadata:
  name: pokeapi-mongodb
type: Opaque
stringData:
  MONGODB_URI: mongodb+srv://usuario:PASSWORD@host/...  # pon TU URI real (no la commitees)
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: pokeapi
spec:
  replicas: 1
  selector:
    matchLabels: { app: pokeapi }
  template:
    metadata:
      labels: { app: pokeapi }
    spec:
      containers:
        - name: pokeapi
          image: ghcr.io/kenesparta/pokeapi:latest
          ports:
            - { name: http, containerPort: 3000 }
          env:
            - name: REDIS_URL
              valueFrom:
                secretKeyRef: { name: pokeapi-redis, key: REDIS_URL }
            - name: MONGODB_URI
              valueFrom:
                secretKeyRef: { name: pokeapi-mongodb, key: MONGODB_URI }
          # /vivo no toca Redis/Mongo: una caída de una dependencia no reinicia
          # el pod ni lo saca del Service (la app degrada, no cae).
          readinessProbe:
            httpGet: { path: /vivo, port: http }
          livenessProbe:
            httpGet: { path: /vivo, port: http }
            initialDelaySeconds: 15
---
apiVersion: v1
kind: Service
metadata:
  name: pokeapi
spec:
  selector: { app: pokeapi }
  ports:
    - { name: http, port: 3000, targetPort: http }
```

```bash
kubectl apply -f pokeapi.yaml
kubectl port-forward svc/pokeapi 3000:3000    # → http://localhost:3000
```

Para exponerla fuera del cluster añade un `Service` `type: LoadBalancer` (o un
Ingress), como hace [`k8s/60-pokeapi.yaml`](../k8s/60-pokeapi.yaml).

### Actualizar la imagen

`:latest` no se re-baja solo. Para traer la última:

```bash
kubectl -n prometheus-demo rollout restart deploy/pokeapi
```

O fija un tag de commit concreto (recomendado en prod):

```bash
kubectl -n prometheus-demo set image deploy/pokeapi \
  pokeapi=ghcr.io/kenesparta/pokeapi:<sha12>
```

## Variables

Se pasan con `-e` (Docker) o en `env:` (K8s).

| Variable | ¿Oblig.? | Default | Qué es |
|---|---|---|---|
| `REDIS_URL` | **sí** | — | `redis://…` o `rediss://…` (TLS); sesiones y caché |
| `MONGODB_URI` | **sí** | — | `mongodb://…` o `mongodb+srv://…` (Atlas); usuarios |
| `MONGODB_DB` | no | `pokeapi` | Nombre de la base de datos en Mongo |
| `ADMIN_PASSWORD` | no | `123` | Password del usuario `admin` sembrado al arrancar |
| `POKEAPI_URL_BASE` | no | `https://pokeapi.co/api/v2` | Base de la PokeAPI |
| `SESION_TTL_SEGUNDOS` | no | `86400` | TTL (deslizante) de las sesiones en Redis |
| `CACHE_TTL_SEGUNDOS` | no | `600` | TTL del caché de fichas en Redis |
| `RUST_LOG` | no | `info` | Nivel de logs (p. ej. `info,backend=debug`) |

Si falta `REDIS_URL` o `MONGODB_URI`, el contenedor arranca y sale con el error
`la variable REDIS_URL no está definida` (o `MONGODB_URI`, respectivamente).
