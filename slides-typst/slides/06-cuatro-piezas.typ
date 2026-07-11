// 3 · Las 4 piezas (esta diapositiva vuelve al final como recap)
#import "../theme.typ": *

#slide("¿Suena complejo? Son sólo 4 piezas.", valign: horizon)[
  #pipeline((
    ("Exporters", "exponen las métricas"),
    ("El servidor", "las recolecta (scrape) y guarda"),
    ("PromQL", "te deja preguntarles cosas"),
    ("Alertmanager", "te avisa cuando algo va mal"),
  ))
]
