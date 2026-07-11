// 4 · QR de Grafana (para curiosos). El tablero bonito, en acceso anónimo de
// solo lectura para el público.
//
// ⚠️ El QR (../assets/qr-grafana.svg) y la IP son los de HOY: regéneralos
// cuando cambie la IP pública del LoadBalancer (ver theme.typ).
#import "../theme.typ": *

#dark-slide[
  #v(1fr)
  #grid(
    columns: (auto, 1fr),
    column-gutter: 46pt,
    align: horizon,
    block(fill: white, radius: 18pt, inset: 16pt, image("../assets/qr-grafana.svg", width: 300pt)),
    [
      #text(size: 13pt, weight: 700, tracking: 0.2em, fill: prom-orange)[SI TE PICA LA CURIOSIDAD]
      #v(18pt)
      #text(size: 44pt, weight: 700, fill: white)[Grafana]
      #v(14pt)
      #text(size: 25pt, weight: 700, font: mono, fill: prom-orange)[http://34.172.120.142]
      #v(18pt)
      #text(size: 19pt, fill: dark-muted)[
        El tablero, en #text(fill: dark-fg)[solo lectura]. \
        Las mismas métricas, ya dibujadas.
      ]
    ],
  )
  #v(1fr)
]
