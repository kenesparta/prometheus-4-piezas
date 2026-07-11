// 9 · Recap — las 4 piezas (misma tarjeta que la diapositiva 3, en corto)
#import "../theme.typ": *

#slide("Recap — las 4 piezas", valign: horizon)[
  #pipeline((
    ("Exporters", "exponen"),
    ("Servidor", "recolecta y guarda"),
    ("PromQL", "pregunta"),
    ("Alertmanager", "avisa"),
  ))
  #v(34pt)
  #align(center, text(size: 17pt, fill: muted)[
    Y encima: *Grafana* (dashboards), *OpenTelemetry* (instrumentación, ya graduado en la CNCF).
  ])
]
