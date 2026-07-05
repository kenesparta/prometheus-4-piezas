# Runbook de la demo — Prometheus en 4 piezas

Guía operativa **paso a paso** (qué teclear, qué señalar). El guion narrativo
vive en `guion-prometheus-4-piezas.md`; aquí está la mecánica. La demo corre en
un cluster Kubernetes (GKE) y se accede vía `port-forward`, así que las URLs son
las mismas `localhost:*` de siempre.

---

## Antes de la charla (la noche anterior y 10 min antes)

- [ ] `kubectl config current-context` apunta al cluster correcto.
- [ ] `kubectl apply -f k8s/` aplicado y `kubectl -n prometheus-demo get pods` todo **Running**.
- [ ] Desplegado **5–10 min antes** para que haya datos (las queries `rate(...[1m])` lo necesitan).
- [ ] `./k8s/port-forward.sh` corriendo en una terminal dedicada.
- [ ] Pestañas preabiertas: `localhost:9090` · `localhost:9100/metrics` · `localhost:9093`.
- [ ] Queries de PromQL copiadas (sección de abajo) en un bloc de notas.
- [ ] Terminal con fuente 18–20pt y alto contraste.
- [ ] **Plan B:** capturas de cada paso por si falla la red/el cluster.

> Regla de oro: nada que tarde más de 10 s en arrancar se arranca en vivo. Todo
> precargado o a un comando de distancia.

---

## DEMO 1 · Exporters — ¿de dónde salen las métricas?

**Abrir en el navegador:**
```
http://localhost:9100/metrics
```

**Qué señalar:**
- Es sólo texto. Cada línea: `nombre{etiquetas} valor`.
- Leer una en voz alta, p. ej. `node_memory_MemAvailable_bytes`.
- "Esto es todo lo que hace un exporter: una página de texto con números. Nada mágico."
- (Modelo **pull**: Prometheus vendrá a buscar esto; la app no lo envía.)

---

## DEMO 2 · El servidor — el recolector

**Mostrar el cableado** (`k8s/20-prometheus-config.yaml`, clave `prometheus.yml`):
el job `node` apunta a `node-exporter:9100`. "Aquí le decimos a Prometheus a
dónde ir y cada cuánto (15 s). Ese es el cable."

**Abrir:**
```
http://localhost:9090/targets
```

**Qué señalar:**
- Los targets `node` y `prometheus` en estado **UP** (verde). "Ya está recolectando."
- (Opcional, si sobra tiempo) Mostrar un target DOWN — ver "Trucos" abajo.

---

## DEMO 3 · PromQL — preguntarle a los datos

Pestaña **Graph**: `http://localhost:9090/graph`. Pegar las queries una a una y
alternar entre **Table** y **Graph**.

```promql
node_memory_MemAvailable_bytes
```
> El valor crudo, igual que en /metrics, pero ahora guardado en el tiempo.

```promql
node_cpu_seconds_total{mode="idle"}
```
> Las etiquetas (entre llaves) filtran. Aquí, sólo el CPU en reposo.

```promql
rate(node_cpu_seconds_total{mode="system"}[1m])
```
> `rate()` = qué tan rápido crece algo. El corazón de PromQL: de "cuánto" a "qué tan rápido".

```promql
100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[1m])) * 100)
```
> Una métrica de verdad: el % de uso de CPU. De números sueltos a un dashboard real.

**Clavar:** "Con `nombre`, `{etiquetas}`, `rate()` y una agregación cubres la mayoría del día a día."

---

## DEMO 4 · Alertmanager — que el sistema te avise

**Mostrar la regla** (`k8s/20-prometheus-config.yaml`, clave `alert.rules.yml`):
el `expr` de `CPUAlto` es la **misma query** de la Demo 3 con un umbral. "Una
alerta no es nada nuevo: es una pregunta con un umbral."

**Abrir:**
```
http://localhost:9090/alerts
```

**Qué señalar:**
- `CPUAlto` (umbral > 80%) en **Inactive** (verde): el sistema está sano.
- `CPUAltoDemo` (umbral > 1%, a propósito) en **Pending → Firing** (rojo) en ~30 s.
  Efecto visual garantizado, sin tocar nada en vivo.

**Mostrar que la alerta llega a Alertmanager:**
```
http://localhost:9093
```
> Ahí estaría el enrutamiento real (Slack, email, PagerDuty). Cierra el círculo:
> de mirar dashboards → a que el sistema hable solo.

---

## Trucos en vivo (opcionales)

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
| Pod en `ImagePullBackOff` | Tag de imagen no disponible → ajusta versión (ver `k8s/README.md`). |
| `/targets` con `node` DOWN | El Service `node-exporter` no resuelve o el pod no está Ready: `kubectl -n prometheus-demo get pods`. |
| `rate()` devuelve vacío | Aún no hay 1–2 min de datos. Espera o despliega antes. |
| `CPUAltoDemo` no dispara | Idem: necesita datos. Confirma que `node` está UP. |
| port-forward se cae | Reabre con `./k8s/port-forward.sh`; revisa `/tmp/pf-*.log`. |
| La alerta no llega a `:9093` | El pod `alertmanager` no está Ready, o `alerting.alertmanagers` mal apuntado. |

## Limpieza

```bash
kubectl delete -f k8s/
```
