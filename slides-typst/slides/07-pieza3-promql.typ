// 7 · Pieza 3 — PromQL (el momento "ajá"; no recortar por tiempo)
#import "../theme.typ": *

#pieza(
  3,
  "PromQL",
  "Preguntarle a tus datos",
)[
  De lo simple a lo útil en 4 pasos:

  #v(10pt)
  #promql-panel((
    ("node_memory_MemAvailable_bytes", "el valor crudo"),
    ("node_cpu_seconds_total{mode=\"idle\"}", "filtrar con etiquetas"),
    ("rate(node_cpu_seconds_total{mode=\"system\"}[1m])", "rate(): qué tan rápido"),
    ("100 - (avg(rate(node_cpu_seconds_total{mode=\"idle\"}[1m])) * 100)", "% de uso de CPU"),
  ))
  #v(1fr)
  #callout[`nombre` + `{etiquetas}` + `rate()` + agregación = el 80% del día a día.]
]
