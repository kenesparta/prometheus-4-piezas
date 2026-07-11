// 10 · Para seguir
#import "../theme.typ": *

#slide("Para seguir")[
  #grid(
    columns: (1fr, 1fr),
    rows: (1fr, 1fr),
    gutter: 14pt,
    seguir-card("DOCUMENTACIÓN OFICIAL", [*prometheus.io*]),
    seguir-card("EXPORTERS", [`node_exporter` y exporters de la comunidad]),
    seguir-card("VISUALIZACIÓN", [*Grafana*]),
    seguir-card("EN KUBERNETES", [busca *kube-prometheus-stack*]),
  )
]
