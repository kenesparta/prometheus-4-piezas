// 2 · QR de la app (uno por página). Se deja al inicio para que el público
// abra la app desde el móvil ya: el tráfico que generan durante la charla
// alimenta las queries y enciende las alertas del bloque de demo.
//
// ⚠️ El QR (../assets/qr-app.svg) y la IP de abajo son los de HOY. Con cada
// cluster cambia la IP pública: regenera el SVG y actualiza el texto (mismo
// procedimiento que `url-app` en theme.typ).
#import "../theme.typ": *

#dark-slide[
  #v(1fr)
  #grid(
    columns: (auto, 1fr),
    column-gutter: 46pt,
    align: horizon,
    block(fill: white, radius: 18pt, inset: 16pt, image("../assets/qr-app.svg", width: 300pt)),
    [
      #text(size: 13pt, weight: 700, tracking: 0.2em, fill: prom-orange)[ESCANEA Y ENTRA]
      #v(18pt)
      #text(size: 44pt, weight: 700, fill: white)[La app]
      #v(14pt)
      #text(size: 25pt, weight: 700, font: mono, fill: prom-orange)[http://34.55.181.146]
      #v(18pt)
      #text(size: 19pt, fill: dark-muted)[
        Créate un usuario y busca un Pokémon. \
        Usuario #text(font: mono, fill: dark-fg)[admin] / #text(font: mono, fill: dark-fg)[123], o el tuyo.
      ]
    ],
  )
  #v(1fr)
]
