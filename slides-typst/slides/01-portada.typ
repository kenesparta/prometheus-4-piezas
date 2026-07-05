// 1 · Portada
#import "../theme.typ": *

#dark-slide[
  #v(1fr)
  #piezas-motif(size: 13pt, on-dark: true)
  #v(30pt)
  #text(size: 13pt, weight: 700, tracking: 0.2em, fill: prom-orange)[PONENCIA CLOUD NATIVE]
  #v(24pt)
  #text(size: 46pt, weight: 700, fill: white)[Si no lo mides, \ no lo controlas]
  #v(28pt)
  #text(size: 26pt, weight: 700, fill: prom-orange)[Prometheus en 4 piezas]
  #v(1fr)
  #line(length: 100%, stroke: 1pt + rgb("#272B33"))
  #v(16pt)
  #text(size: 14pt, fill: dark-muted)[\~30 min · demo en vivo #h(1fr) prometheus.io]
]
