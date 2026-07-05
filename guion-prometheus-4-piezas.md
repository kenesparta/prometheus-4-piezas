# Si no lo mides, no lo controlas: Prometheus en 4 piezas

**Formato:** Ponencia Cloud Native · ~30 min (incl. preguntas) · Demo en vivo
**Público:** Principiantes totales (sin experiencia previa con Prometheus)
**Demo:** corre sobre un cluster Kubernetes (GKE). Mecánica detallada en `DEMO.md`; manifiestos en `k8s/`.

---

## Resumen

En un mundo de microservicios, contenedores efímeros y cargas de IA, saber qué pasa dentro de tus sistemas dejó de ser opcional. Prometheus es la herramienta graduada de la CNCF que respondió a esa necesidad y hoy es el punto de partida de cualquier stack Cloud Native. Esta charla desmitifica Prometheus reduciéndolo a sus cuatro piezas esenciales —exporters, el servidor de scraping, PromQL y Alertmanager— y demuestra que con esos cuatro conceptos ya entiendes el 80% de cómo funciona. Pasaremos de un sistema "ciego" a uno que se mide, se consulta y se alerta a sí mismo.

---

## Antes de empezar: checklist técnico

Prepara esto la noche anterior y deja todo precargado para no perder tiempo en vivo. La mecánica completa está en `DEMO.md`.

- [ ] `kubectl config current-context` apunta al cluster (GKE) correcto.
- [ ] `kubectl apply -f k8s/` aplicado; `kubectl -n prometheus-demo get pods` todo **Running**.
- [ ] Desplegado **5-10 min antes** de la charla (las queries `rate(...[1m])` y la alerta necesitan datos).
- [ ] `./k8s/port-forward.sh` corriendo en una terminal dedicada (abre 9090/9100/9093 en localhost).
- [ ] Pestañas del navegador preabiertas: `localhost:9090` (Prometheus), `localhost:9100/metrics` (exporter), `localhost:9093` (Alertmanager).
- [ ] Terminal con fuente grande (18-20pt mínimo) y tema de alto contraste.
- [ ] Queries de PromQL copiadas en un bloc de notas, listas para pegar.
- [ ] **Plan B:** capturas de pantalla de cada paso por si la demo (o la red al cluster) falla.

> **Regla de oro de la demo:** nada que tarde más de 10 segundos en arrancar debería arrancarse en vivo. Todo desplegado y con port-forward antes de empezar; en vivo sólo se abren pestañas y se pegan queries.

---

## Estructura general

| # | Bloque | Tiempo | Demo |
|---|--------|--------|------|
| 0 | Apertura: el problema | 3 min | — |
| 1 | Pieza 1 — Exporters | 5 min | Sí |
| 2 | Pieza 2 — El servidor | 5 min | Sí |
| 3 | Pieza 3 — PromQL | 7 min | Sí |
| 4 | Pieza 4 — Alertmanager | 5 min | Sí |
| 5 | Cierre: dónde encaja | 2 min | — |
| — | Preguntas | 3 min | — |

> Total ~30 min. Margen frente a los 25-28 min originales: va sobre todo a PromQL (pieza 3, la de mayor valor) y a dejar aire para preguntas.

---

## 0 · Apertura — El problema (2-3 min)

**Objetivo:** enganchar antes de explicar nada técnico.

**Guion hablado (idea, no leer literal):**

> "Imaginen las 2 de la madrugada. Su aplicación está corriendo, los servidores encendidos, todo 'verde'. Pero el dashboard está en blanco. No saben si la app está sana, si va a caer en cinco minutos, o si ya cayó. El sistema funciona... pero es invisible. Eso es lo que pasa cuando tienes infraestructura pero no tienes observabilidad."

**Puntos a clavar:**
- Monitoreo tradicional (¿está vivo el servidor?) ≠ observabilidad (¿qué está pasando *dentro*?).
- En Cloud Native los contenedores son efímeros: nacen y mueren en segundos. No puedes entrar a "mirar" cada uno.
- La CNCF lo resume así: la IA está subiendo el listón de la confiabilidad, y la pregunta dura es si puedes medir lo que corre en producción.
- Prometheus es la respuesta estándar: graduado en la CNCF, el segundo proyecto en lograrlo después de Kubernetes.

**Transición:** "Suena complejo, pero Prometheus en realidad son solo 4 piezas. Vamos una por una."

---

## 1 · Pieza 1 — Exporters: ¿de dónde salen las métricas? (4-5 min)

**Idea central:** Antes de medir algo, ese algo tiene que *exponer* sus números. Eso lo hace un exporter.

**Conceptos a explicar:**
- Un exporter expone métricas en un endpoint HTTP, normalmente `/metrics`, en texto plano.
- **Modelo pull, no push:** Prometheus *va a buscar* los datos; la app no los envía. Esto es contraintuitivo para principiantes — recalcarlo.
- Hay exporters para todo: bases de datos, el sistema operativo (`node_exporter`), apps propias instrumentadas con librerías.

**Gancho narrativo (opcional, 30s):** Dos proyectos de la CNCF pueden estar instalados perfectamente y aun así ser invisibles entre sí si nadie los "cablea". El servidor no adivina dónde están las métricas: hay que decírselo. Esa es la pieza que viene después.

### DEMO 1

> Ya desplegado en el cluster (`k8s/10-node-exporter.yaml`) y expuesto vía port-forward. En vivo sólo abrir el navegador. Pasos en `DEMO.md`.

Abrir en el navegador:

```
http://localhost:9100/metrics
```

**Qué señalar en pantalla:**
- Es solo texto. Cada línea es una métrica: `nombre{etiquetas} valor`.
- Buscar una concreta y leerla en voz alta, ej: `node_memory_MemAvailable_bytes`.
- "Esto es todo lo que hace un exporter: una página de texto con números actualizados. Nada mágico."

**Transición:** "Estos números están aquí, pero nadie los guarda ni los grafica. Necesitamos a quien los recolecte."

---

## 2 · Pieza 2 — El servidor de Prometheus: el recolector (4-5 min)

**Idea central:** El servidor de Prometheus hace *scrape*: visita cada endpoint `/metrics` cada cierto tiempo y guarda los valores como series temporales.

**Conceptos a explicar:**
- **Scrape:** Prometheus pregunta "¿cuánto vale esto ahora?" cada X segundos (típico: 15s).
- **Series temporales:** cada métrica se guarda con su marca de tiempo. Así puedes ver la evolución, no solo el valor actual.
- **Targets:** la lista de endpoints a scrapear. Esto es el "cableado" del que hablamos.
- Todo se configura en un archivo `prometheus.yml`.

**Mostrar el archivo de config (ya escrito):**

```yaml
# prometheus.yml (k8s/20-prometheus-config.yaml)
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'node'
    static_configs:
      - targets: ['node-exporter:9100']   # el exporter de la pieza 1 (DNS del Service)
```

> Señalar: "Aquí le decimos a Prometheus: ve a `node-exporter:9100` cada 15 segundos. Ese es el cable que conecta el exporter con el recolector."

### DEMO 2

> Prometheus ya corre en el cluster (`k8s/22-prometheus.yaml`). En vivo sólo abrir el navegador.

Abrir:

```
http://localhost:9090/targets
```

**Qué señalar:**
- Los targets `node` y `prometheus` aparecen en estado **UP** (verde). "Prometheus ya está recolectando."
- Si tienes tiempo: cambiar el target a un puerto inexistente, recargar la config y mostrar el estado **DOWN** (ver "Trucos en vivo" en `DEMO.md`). Ilustra el "impuesto de integración" de forma visual.

**Transición:** "Ya tenemos datos guardándose. Pero un montón de números no sirve de nada si no podemos hacerles preguntas. Ahí entra el lenguaje de Prometheus."

---

## 3 · Pieza 3 — PromQL: hacer preguntas a tus datos (5-6 min)

**Idea central:** PromQL es el lenguaje para consultar las métricas. Es el momento "ajá" de la charla: los números crudos se vuelven respuestas útiles.

**Concepto clave para principiantes:** No te asustes con la sintaxis. Vamos de lo simple a lo útil en 4 pasos.

### DEMO 3

Usar la pestaña **Graph** de Prometheus (`http://localhost:9090/graph`). Pegar las queries una a una.

**Query 1 — la más simple: el valor actual**

```promql
node_memory_MemAvailable_bytes
```

> "Esto es el valor crudo, igual que en el endpoint /metrics, pero ahora guardado en el tiempo. Cambia a la pestaña Graph y verás la línea."

**Query 2 — filtrar con etiquetas**

```promql
node_cpu_seconds_total{mode="idle"}
```

> "Las etiquetas (lo que va entre llaves) te dejan filtrar. Aquí, solo el CPU en reposo."

**Query 3 — `rate()`: la función estrella**

```promql
rate(node_cpu_seconds_total{mode="system"}[1m])
```

> "`rate()` calcula qué tan rápido crece algo. Esto es el corazón de PromQL: pasar de 'cuánto' a 'qué tan rápido'. Casi todo lo útil usa rate."

**Query 4 — agregar varias series**

```promql
100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[1m])) * 100)
```

> "Y esto ya es una métrica de verdad: el porcentaje de uso de CPU. De números sueltos a un dashboard real."

**Qué clavar:** "Con `nombre`, `{etiquetas}`, `rate()` y una agregación, ya cubres la mayoría de lo que harás a diario."

**Transición:** "Pero nadie va a quedarse mirando estos gráficos las 24 horas. Queremos que el sistema nos avise solo. Última pieza."

---

## 4 · Pieza 4 — Alertmanager: que el sistema te avise (3-4 min)

**Idea central:** Una alerta es una query de PromQL con una condición. Cuando se cumple, Alertmanager se encarga de notificar.

**Conceptos a explicar:**
- En Prometheus defines **reglas de alerta**: "si esta query supera X durante Y tiempo, dispara."
- **Alertmanager** es una pieza separada que recibe las alertas y las enruta: email, Slack, PagerDuty, etc. También agrupa y silencia.
- Cierra el círculo: de mirar dashboards → a que el sistema hable solo.

**Mostrar una regla (ya escrita):**

```yaml
# alert.rules.yml (k8s/20-prometheus-config.yaml)
groups:
  - name: ejemplo
    rules:
      - alert: CPUAlto
        expr: 100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[1m])) * 100) > 80
        for: 1m
        labels:
          severity: warning
        annotations:
          summary: "Uso de CPU por encima del 80%"
```

> Señalar: "Fíjense que el `expr` es la misma query de PromQL de hace un minuto. Una alerta no es nada nuevo: es una pregunta con un umbral."

### DEMO 4

Abrir:

```
http://localhost:9090/alerts
```

**Qué señalar:**
- `CPUAlto` (umbral > 80%) en **Inactive** (verde): el sistema está sano.
- Junto a ella, `CPUAltoDemo` (mismo `expr` pero umbral > 1%, puesto bajo a propósito) pasa de **Pending** a **Firing** (rojo) en ~30 s. Efecto visual garantizado, **sin tocar nada en vivo** (en K8s recargar un ConfigMap tarda ~1 min y arruinaría el momento).
- Mostrar que la alerta llega a Alertmanager (`localhost:9093`): ahí iría el enrutamiento real (Slack, email, PagerDuty).

**Transición:** "Y con esto cerramos el círculo completo."

---

## 5 · Cierre — Dónde encaja en Cloud Native (2 min)

**Recapitular las 4 piezas (slide de resumen):**

1. **Exporters** → exponen las métricas.
2. **Servidor** → las recolecta (scrape) y las guarda.
3. **PromQL** → te deja preguntarles cosas.
4. **Alertmanager** → te avisa cuando algo va mal.

**Mensaje de cierre:**
- Prometheus no vive solo: es la base sobre la que se apoyan **Grafana** (dashboards bonitos), **OpenTelemetry** (que se graduó en la CNCF y se está volviendo el estándar de instrumentación) y el resto del stack.
- "Hoy vieron lo básico, pero lo básico es el 80%. Con estas 4 piezas ya pueden empezar a observar sus propios sistemas."

**Recursos para seguir (slide final):**
- Documentación oficial: prometheus.io
- `node_exporter` y exporters de la comunidad.
- Grafana para visualización.
- Buscar "kube-prometheus-stack" cuando salten a Kubernetes.

**Cierre de una línea:**

> "Si no lo mides, no lo controlas. Ahora ya saben cómo medirlo."

---

## Notas de tiempo y recortes

- Si vas **apretado de tiempo**: PromQL (pieza 3) es donde más valor tiene detenerse — no lo recortes. Alertmanager (pieza 4) es lo primero que reduzco a una demo rápida sin entrar en enrutamiento.
- Si te **sobra tiempo**: en la pieza 2, muestra el estado DOWN de un target para reforzar el concepto de "cableado".
- **Preguntas:** deja 2-3 min al final. Las más comunes de principiantes: "¿pull vs push por qué?", "¿esto reemplaza a Grafana?" (no, se complementan), "¿funciona fuera de Kubernetes?" (sí).
