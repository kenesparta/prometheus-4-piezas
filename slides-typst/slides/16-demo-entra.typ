// 13 · Arranca el bloque de demo (10 min). Esta diapositiva se deja en
// pantalla ~1 min mientras el público entra a la app: el tráfico que generan
// es el que se verá en las queries y el que enciende las alertas.
//
// ⚠️ La URL se edita en theme.typ (#let url-app): la IP del LoadBalancer
// cambia con cada cluster. Sin raw inline aquí: su fondo claro no pega en oscuro.
#import "../theme.typ": *

#dark-slide[
  #v(1fr)
  #text(size: 13pt, weight: 700, tracking: 0.2em, fill: prom-orange)[AHORA, USTEDES]
  #v(24pt)
  #text(size: 42pt, weight: 700, fill: white)[Entren a la app]
  #v(30pt)
  #text(size: 33pt, weight: 700, font: mono, fill: prom-orange, url-app)
  #v(30pt)
  #text(size: 18pt, fill: dark-muted)[
    Usuario #text(font: mono, fill: dark-fg)[admin] / #text(font: mono, fill: dark-fg)[123],
    o regístrate con lo que quieras. \
    Busca pokémon. Falla el login a propósito. #text(fill: dark-fg)[Rompe algo.]
  ]
  #v(1fr)
]
