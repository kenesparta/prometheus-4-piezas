// 5 · Pieza 2 — El servidor
#import "../theme.typ": *

#pieza(
  2,
  "El servidor",
  "El recolector",
  demo: demo-strip[`localhost:9090/targets` #h(10pt) → #h(10pt) target #badge("UP", ok-green)],
)[
  - *Scrape:* "¿cuánto vale esto ahora?" cada 15 s.
  - *Series temporales:* cada valor con su marca de tiempo.
  - *Targets:* la lista de endpoints = el "cableado" (`prometheus.yml`).
]
