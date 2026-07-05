// 4 · Pieza 1 — Exporters
#import "../theme.typ": *

#pieza(
  1,
  "Exporters",
  "¿De dónde salen las métricas?",
  demo: demo-strip[`localhost:9100/metrics` #h(10pt) → #h(10pt) `nombre{etiquetas} valor`],
)[
  - Un exporter expone métricas en HTTP, normalmente `/metrics`, en texto plano.
  - *Modelo pull, no push:* Prometheus _va a buscar_ los datos.
  - Hay exporters para todo: SO (`node_exporter`), bases de datos, apps propias.
]
