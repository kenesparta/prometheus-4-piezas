# Runbook de la demo — Prometheus en 4 piezas

Guía operativa **paso a paso** (qué teclear, qué señalar). El guion narrativo
vive en `guion-prometheus-4-piezas.md`; aquí está la mecánica.

La charla son 20 min de diapositivas **sin navegador** y luego un bloque único de
**10 min de demo**. La demo corre en un cluster Kubernetes (GKE): el presentador
llega vía `port-forward` (URLs `localhost:*` de siempre) y **el público entra a la
app por su IP pública** desde el móvil.

---

## Antes de la charla (la noche anterior y 10 min antes)

- [ ] `kubectl config current-context` apunta al cluster correcto.
- [ ] `./k8s/montar-charla.sh` — idempotente: namespace + secrets + manifiestos,
      espera pods y muestra las IPs públicas. `--estado` sólo muestra el estado.
- [ ] `kubectl -n prometheus-demo get pods` todo **Running**.
- [ ] Desplegado **5–10 min antes** para que haya datos (las queries `rate(...[1m])` lo necesitan).
- [ ] **La IP pública de la app**, anotada:
      ```bash
      kubectl -n prometheus-demo get svc pokeapi-publico \
        -o jsonpath='{.status.loadBalancer.ingress[0].ip}'
      ```
- [ ] **URL puesta en las diapositivas** (cambia con cada cluster):
      - `slides-typst/theme.typ` → `#let url-app = "http://..."`, y recompilar (`make -C slides-typst`).
      - `slides.md` → la slide "Ahora, ustedes" (`http://<IP-DE-LA-APP>`).
- [ ] La app responde: `curl -s http://<IP>/salud | jq` → `redis: ok`, `mongo: ok`.
- [ ] `./k8s/port-forward.sh` corriendo en una terminal dedicada.
- [ ] Pestañas preabiertas: `localhost:9090` · `localhost:9100/metrics` ·
      `localhost:3000/metrics` · `localhost:9093`.
- [ ] Queries de PromQL copiadas (sección de abajo) en un bloc de notas.
- [ ] Terminal con fuente 18–20pt y alto contraste.
- [ ] **Plan B:** capturas de cada acto por si falla la red/el cluster.

> Regla de oro: nada que tarde más de 10 s en arrancar se arranca en vivo. Todo
> precargado o a un comando de distancia.

> **Plan B de red:** si `kubectl` se pone lento pero hay internet, Prometheus y
> Grafana también tienen IP pública (`prometheus-publico`, `grafana`). Ténlas
> anotadas junto a la de la app; `montar-charla.sh --estado` las imprime.

---

## ACTO 0 · "Entren a la app" (~1 min)

Proyectar la diapositiva 13 con la URL. Mientras la sala teclea:

> "Regístrense, o usen `admin` / `123`. Busquen un pokémon. Busquen el mismo dos
> veces y miren la latencia. Y equivóquense de contraseña a propósito."

**No avanzar** hasta oír teclado. Ese tráfico es el combustible de los actos 3 y 4.

---

## ACTO 1 · Exporters — ¿de dónde salen las métricas? (~2 min)

**Abrir:**
```
http://localhost:9100/metrics
```

- Es sólo texto. Cada línea: `nombre{etiquetas} valor`.
- Leer una en voz alta, p. ej. `node_memory_MemAvailable_bytes`.
- (Modelo **pull**: Prometheus vendrá a buscar esto; la app no lo envía.)

**Y ahora el bueno** — la app, que es un exporter hecho en casa:
```
http://localhost:3000/metrics
```

- Buscar `pokeapi_http_peticiones_total` y **recargar**: el contador sube.
- "Cada incremento es un clic de alguien en esta sala."

---

## ACTO 2 · El servidor — el recolector (~1 min)

**Abrir:**
```
http://localhost:9090/targets
```

- Los targets `node`, `prometheus` y `pokeapi` en **UP** (verde).
- "Para Prometheus, nuestra app es un target más. El cable funciona."

---

## ACTO 3 · PromQL — preguntarle a los datos (~3 min)

Pestaña **Graph**: `http://localhost:9090/graph`. Pegar las queries una a una y
alternar entre **Table** y **Graph**.

**La escalera (rápido, ya se explicó en la charla):**

```promql
node_memory_MemAvailable_bytes
node_cpu_seconds_total{mode="idle"}
rate(node_cpu_seconds_total{mode="system"}[1m])
100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[1m])) * 100)
```

**Las de la app (despacio — aquí está el "ajá"):**

```promql
sum(rate(pokeapi_http_peticiones_total[1m])) by (rol)
```
> La curva sube con el público. Aparecen `anonimo`, `VISITOR`, `ADMIN`.

```promql
sum(rate(pokeapi_pokemon_consultas_total[5m])) by (origen)
```
> `cache` vs `api`: la primera consulta va a la PokeAPI pública, las demás salen
> de Redis. El caché gana por goleada.

```promql
pokeapi_sesiones_activas
```
> "Esos son ustedes."

**Bonus si sobra tiempo** (latencia p95 — misma receta, un ingrediente más):
```promql
histogram_quantile(0.95, sum(rate(pokeapi_http_duracion_segundos_bucket[5m])) by (le))
```

---

## ACTO 4 · Alertmanager — que el sistema te avise (~2 min)

**Abrir:**
```
http://localhost:9090/alerts
```

**Qué señalar, en este orden:**
- `CPUAlto` (umbral > 80%) en **Inactive** (verde): el sistema está sano.
- `CPUAltoDemo` (umbral > 1%, a propósito) en **Firing** (rojo).
- `PokeapiTraficoAltoDemo` en **Firing**: *esa la encendieron ellos*.
- Pedir a la sala: **"tecleen mal la contraseña, todos, durante un minuto"** →
  `PokeapiFuerzaBrutaDemo` pasa a **Pending** y luego a **Firing**.

> ⚠️ `increase(...[1m])` necesita fallos **sostenidos ~1 min**, no una ráfaga de
> golpe: un burst justo cuando la serie nace no ve el salto 0→N y no dispara.
> Habla mientras tanto; si la sala no colabora, usa el bucle de "Trucos".

**Cerrar en Alertmanager:**
```
http://localhost:9093
```
> Ahí estaría el enrutamiento real (Slack, email, PagerDuty). Cierra el círculo:
> de mirar dashboards → a que el sistema hable solo.

---

## Trucos en vivo (opcionales)

**Generar tráfico si la sala no colabora** (dilo en voz alta: "hago trampa, pero
la métrica no miente"):
```bash
while true; do
  for p in pikachu eevee bulbasaur charmander snorlax; do
    curl -s "localhost:3000/api/pokemon/$p" >/dev/null
  done
  sleep 1
done
```

**Forzar `PokeapiFuerzaBrutaDemo`** (>5 passwords incorrectos en 1 min):
```bash
while true; do
  curl -s -X POST localhost:3000/api/login \
    -H 'content-type: application/json' \
    -d '{"nombre":"admin","password":"incorrecto"}' >/dev/null
  sleep 2
done
```

**Tirar Redis para encender `PokeapiSinRedis`:** basta con romper el Secret y
reiniciar el pod. Ojo: durante la caída **no se puede iniciar sesión** (las
sesiones viven en Redis), pero la app sigue en pie, `up == 1` y sirviendo
`/metrics` — que es justo lo que se quiere enseñar. Las probes apuntan a `/vivo`,
no a `/salud`, precisamente para esto.

**Target DOWN (refuerza el "cableado"):** edita el job `node` a un puerto
inexistente y recarga:
```bash
# cambia node-exporter:9100 -> node-exporter:9199 en 20-prometheus-config.yaml
kubectl -n prometheus-demo apply -f k8s/20-prometheus-config.yaml
# ~1 min de propagación del ConfigMap, luego:
curl -X POST http://localhost:9090/-/reload
```
`/targets` mostrará `node` en **DOWN** (rojo). Revierte igual al terminar.

**Bonus app propia:** ver cabecera de `k8s/50-sample-app.yaml`.

---

## Troubleshooting

| Síntoma | Causa probable / arreglo |
|---|---|
| Pod en `ImagePullBackOff` | El package `ghcr.io/kenesparta/pokeapi` dejó de ser público, o el tag no existe (ver `k8s/README.md`). |
| `pokeapi` en `CrashLoopBackOff` al arrancar | Faltan los Secrets: `kubectl apply -f k8s/secrets.local.yaml` (gitignorado, el CI no puede aplicarlo). |
| `pokeapi` en `Error` justo tras montar | MongoDB Atlas (tier gratuito) estaba pausado. **Se auto-cura** en ~1–3 min; espera o reanúdalo en la consola. |
| El pod no toma la imagen nueva | `:latest` no se re-baja solo: `kubectl -n prometheus-demo rollout restart deploy/pokeapi`. |
| Cambiaste un Secret y la app no lo ve | K8s congela las env de Secrets al arrancar el pod: `kubectl -n prometheus-demo delete pod -l app=pokeapi`. |
| `/targets` con `node` o `pokeapi` DOWN | El Service no resuelve o el pod no está Ready: `kubectl -n prometheus-demo get pods`. |
| `rate()` devuelve vacío | Aún no hay 1–2 min de datos. Espera o despliega antes. |
| `CPUAltoDemo` no dispara | Ídem: necesita datos. Confirma que `node` está UP. |
| `PokeapiFuerzaBrutaDemo` no dispara | Los fallos deben ser **sostenidos ~1 min** (ver Acto 4). Usa el bucle de "Trucos". |
| `pokeapi-publico` sin IP externa | El LoadBalancer tarda 1–3 min. `montar-charla.sh --estado` reintenta. |
| port-forward se cae | Reabre con `./k8s/port-forward.sh`; revisa `/tmp/pf-*.log`. |
| La alerta no llega a `:9093` | El pod `alertmanager` no está Ready, o `alerting.alertmanagers` mal apuntado. |

## Limpieza

```bash
kubectl delete namespace prometheus-demo
```
