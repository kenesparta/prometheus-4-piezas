// 14 · El mapa del bloque de demo: las mismas 4 piezas, ahora en vivo y con
// los datos que acaba de generar el público. Mecánica paso a paso en DEMO.md.
#import "../theme.typ": *

#slide("La demo en 4 actos · 10 min", valign: horizon)[
  #pipeline((
    ("Exporter", "/metrics: sus clics, en texto plano"),
    ("Servidor", "/targets: el job pokeapi en UP"),
    ("PromQL", "su tráfico por rol; caché vs API"),
    ("Alertmanager", "/alerts en Firing"),
  ))
  #v(34pt)
  #align(center, text(size: 17pt, fill: muted)[
    Todo lo que están viendo lo generaron *ustedes* en los últimos 5 minutos.
  ])
]
