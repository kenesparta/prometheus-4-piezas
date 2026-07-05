---
marp: true
theme: default
paginate: true
title: "Si no lo mides, no lo controlas: Prometheus en 4 piezas"
---

<!--
Slides de apoyo (Marp). Render:
  npx @marp-team/marp-cli@latest slides.md -o slides.html   # o --pdf
Alineadas con guion-prometheus-4-piezas.md. La carne está en las DEMOS;
estas diapositivas son sólo anclas visuales, no para leer.
-->

# Si no lo mides, no lo controlas
## Prometheus en 4 piezas

Ponencia Cloud Native · ~30 min · demo en vivo

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

<!-- Esta slide vuelve al final como recap. -->

---

# Pieza 1 · Exporters
### ¿De dónde salen las métricas?

- Un exporter expone métricas en HTTP, normalmente `/metrics`, en texto plano.
- **Modelo pull, no push:** Prometheus *va a buscar* los datos.
- Hay exporters para todo: SO (`node_exporter`), bases de datos, apps propias.

**DEMO:** `localhost:9100/metrics` → `nombre{etiquetas} valor`

---

# Pieza 2 · El servidor
### El recolector

- **Scrape:** "¿cuánto vale esto ahora?" cada 15 s.
- **Series temporales:** cada valor con su marca de tiempo.
- **Targets:** la lista de endpoints = el "cableado" (`prometheus.yml`).

**DEMO:** `localhost:9090/targets` → target **UP**

---

## El cable (prometheus.yml)

```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'node'
    static_configs:
      - targets: ['node-exporter:9100']
```

> "Ve a `node-exporter:9100` cada 15 s." Eso es todo.

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

# Pieza 4 · Alertmanager
### Que el sistema te avise

- Una **regla de alerta** = una query de PromQL + un umbral + `for`.
- **Alertmanager** recibe, agrupa, silencia y enruta (Slack, email, PagerDuty).

```yaml
- alert: CPUAlto
  expr: 100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[1m])) * 100) > 80
  for: 1m
```

**DEMO:** `localhost:9090/alerts` → Firing → llega a `localhost:9093`

> El `expr` es la misma query de la Pieza 3. Nada nuevo.

---

## Recap — las 4 piezas

1. **Exporters** → exponen
2. **Servidor** → recolecta y guarda
3. **PromQL** → pregunta
4. **Alertmanager** → avisa

Y encima: **Grafana** (dashboards), **OpenTelemetry** (instrumentación, ya graduado en la CNCF).

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
