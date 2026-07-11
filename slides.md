---
marp: true
theme: default
paginate: true
title: "Si no lo mides, no lo controlas: Prometheus en 4 piezas"
---

<!--
Slides de apoyo (Marp). Render:
  npx @marp-team/marp-cli@latest slides.md -o slides.html   # o --pdf
Alineadas con guion-prometheus-4-piezas.md. La carne está en la DEMO;
estas diapositivas son sólo anclas visuales, no para leer.

Formato: 20 min de charla (diapositivas 1-12) + 10 min de demo en vivo
(diapositivas 13-14). Nada de navegador hasta el bloque de demo.

⚠️ ANTES DE CADA CHARLA: cambiar la URL de la app en la diapositiva "Entren
aquí" — la IP del LoadBalancer cambia con cada cluster:
  kubectl -n prometheus-demo get svc pokeapi-publico
-->

# Si no lo mides, no lo controlas
## Prometheus en 4 piezas

Ponencia Cloud Native · 20 min de charla + 10 de demo en vivo

<!-- Apertura: enganchar, no tecnicismos todavía. -->

---

## Las 2 de la madrugada

- Todo "verde"... pero el dashboard en blanco.
- ¿La app está sana? ¿Va a caer? ¿Ya cayó?
- **Monitoreo** (¿está vivo?) ≠ **observabilidad** (¿qué pasa *dentro*?).
- Contenedores efímeros: nacen y mueren en segundos. No puedes "entrar a mirar".

> Prometheus: proyecto **graduado** de la CNCF — el 2.º después de Kubernetes.

---

## Suena complejo. Son sólo 4 piezas.

1. **Exporters** → exponen las métricas
2. **El servidor** → las recolecta (scrape) y guarda
3. **PromQL** → te deja preguntarles cosas
4. **Alertmanager** → te avisa cuando algo va mal

<!-- Esta slide vuelve al final como recap. Y al final de todo, la demo. -->

---

# Pieza 1 · Exporters
### ¿De dónde salen las métricas?

- Un exporter expone métricas en HTTP, normalmente `/metrics`, en texto plano.
- **Modelo pull, no push:** Prometheus *va a buscar* los datos.
- Hay exporters para todo: SO (`node_exporter`), bases de datos, apps propias.

**EN LA DEMO:** `localhost:9100/metrics` → `nombre{etiquetas} valor`

---

## La app que vamos a observar

**pokeapi** — una pokédex web en Rust + Leptos. La van a usar ustedes.

- **MongoDB** guarda los usuarios · **Redis**, las sesiones y el caché.
- Buscas un pokémon: la 1.ª vez sale de la PokeAPI pública; las siguientes, del **caché en Redis** — y se nota en la latencia.
- Login, registro y roles (`ADMIN` / `EDITOR` / `VISITOR`).
- **Ella misma expone `/metrics`.** Un exporter hecho en casa.

> No es un exporter de juguete: es *tu* app instrumentada. Eso es lo que harás en el trabajo.

---

# Pieza 2 · El servidor
### El recolector

- **Scrape:** "¿cuánto vale esto ahora?" cada 15 s.
- **Series temporales:** cada valor con su marca de tiempo.
- **Targets:** la lista de endpoints = el "cableado" (`prometheus.yml`).

**EN LA DEMO:** `localhost:9090/targets` → targets **UP**

---

## El cable (prometheus.yml)

```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'node'
    static_configs:
      - targets: ['node-exporter:9100']

  - job_name: 'pokeapi'          # nuestra app, un target más
    static_configs:
      - targets: ['pokeapi:3000']
```

> "Ve a estas direcciones cada 15 s." Eso es todo.

---

# Pieza 3 · PromQL
### Preguntarle a tus datos

De lo simple a lo útil en 4 pasos:

```promql
node_memory_MemAvailable_bytes                       # valor crudo
node_cpu_seconds_total{mode="idle"}                  # filtrar con etiquetas
rate(node_cpu_seconds_total{mode="system"}[1m])      # rate(): qué tan rápido
100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[1m])) * 100)  # % CPU
```

> `nombre` + `{etiquetas}` + `rate()` + agregación = el 80% del día a día.

---

## Las mismas 4 herramientas, sobre la app

```promql
sum(rate(pokeapi_http_peticiones_total[1m])) by (rol)      # tráfico por rol
sum(rate(pokeapi_pokemon_consultas_total[5m])) by (origen) # ¿caché o API pública?
pokeapi_sesiones_activas                                   # cuánta gente hay dentro
sum(increase(pokeapi_login_errores_total[1m]))             # logins fallidos
```

> Cambia la app; el lenguaje, no. El **negocio** se pregunta igual que el CPU.

---

# Pieza 4 · Alertmanager
### Que el sistema te avise

- Una **regla de alerta** = una query de PromQL + un umbral + `for`.
- **Alertmanager** recibe, agrupa, silencia y enruta (Slack, email, PagerDuty).

```yaml
- alert: CPUAlto
  expr: 100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[1m])) * 100) > 80
  for: 1m
```

> El `expr` es la misma query de la Pieza 3. Nada nuevo.

---

## Las alertas de la app

| Alerta | Qué vigila |
|---|---|
| `PokeapiCaida` | la app no responde al scrape (target DOWN) |
| `PokeapiSinRedis` | perdió el caché y las sesiones |
| `PokeapiSinMongo` | perdió la base de usuarios |
| `PokeapiFuerzaBrutaDemo` | demasiados passwords incorrectos por minuto |

```yaml
- alert: PokeapiFuerzaBrutaDemo
  expr: >-
    sum(increase(pokeapi_login_errores_total{motivo="password_incorrecto"}[1m]))
    > 5
  for: 30s
```

> Hasta una alerta de **seguridad** es lo mismo: una pregunta, un umbral y un `for`.

---

## Recap — las 4 piezas

1. **Exporters** → exponen
2. **Servidor** → recolecta y guarda
3. **PromQL** → pregunta
4. **Alertmanager** → avisa

Y encima: **Grafana** (dashboards), **OpenTelemetry** (instrumentación, ya graduado en la CNCF).

---

<!-- _class: invert -->

# Ahora, ustedes.

## `http://34.55.181.146`

Usuario `admin` / `123` — o regístrate con lo que quieras.

Busca pokémon. Falla el login a propósito. **Rompe algo.**

<!--
⚠️ Sustituir la IP antes de la charla:
   kubectl -n prometheus-demo get svc pokeapi-publico
Dejar esta slide en pantalla ~1 min mientras el público entra: el tráfico que
generan es el que se verá en las queries y el que enciende las alertas.
-->

---

## La demo en 4 actos · 10 min

1. **Exporter** — `/metrics` de la app: sus clics, en texto plano.
2. **Servidor** — `/targets`: el job `pokeapi` en **UP**.
3. **PromQL** — su tráfico por rol; caché vs API pública.
4. **Alertmanager** — `/alerts` en **Firing**. Tecleen mal la contraseña.

> Todo lo que están viendo lo generaron ustedes en los últimos 5 minutos.

---

## Para seguir

- Documentación oficial: **prometheus.io**
- `node_exporter` y exporters de la comunidad
- **Grafana** para visualización
- En Kubernetes: busca **kube-prometheus-stack**

---

# "Si no lo mides, no lo controlas."
## Ahora ya saben cómo medirlo.

¿Preguntas?
