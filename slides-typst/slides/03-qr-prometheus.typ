// 3 · QR de Prometheus (para curiosos). La UI de las 4 piezas: /targets,
// PromQL y /alerts. Solo lectura para el público.
//
// ⚠️ El QR (../assets/qr-prometheus.svg) y la IP son los de HOY: regéneralos
// cuando cambie la IP pública del LoadBalancer (ver theme.typ).
#import "../theme.typ": *

#dark-slide[
  #v(1fr)
  #grid(
    columns: (auto, 1fr),
    column-gutter: 46pt,
    align: horizon,
    block(fill: white, radius: 18pt, inset: 16pt, image("../assets/qr-prometheus.svg", width: 300pt)),
    [
      #text(size: 13pt, weight: 700, tracking: 0.2em, fill: prom-orange)[SI TE PICA LA CURIOSIDAD]
      #v(18pt)
      #text(size: 44pt, weight: 700, fill: white)[Prometheus]
      #v(14pt)
      #text(size: 25pt, weight: 700, font: mono, fill: prom-orange)[http://34.10.208.11]
      #v(18pt)
      #text(size: 19pt, fill: dark-muted)[
        Las métricas crudas: #text(fill: dark-fg)[targets], #text(fill: dark-fg)[PromQL] y #text(fill: dark-fg)[alertas], \
        moviéndose en vivo con lo que ustedes generen.
      ]
    ],
  )
  #v(1fr)
]
