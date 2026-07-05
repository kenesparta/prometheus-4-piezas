// theme.typ — paleta, tipografía y componentes del deck.
// Las diapositivas viven en slides/NN-*.typ e importan esto con:
//   #import "../theme.typ": *
// El punto de entrada (slides.typ) aplica las reglas globales con #show: setup.

// ── Paleta ───────────────────────────────────────────────────────────────
#let prom-orange = rgb("#E6522C") // naranja Prometheus
#let prom-ember = rgb("#C64A22") // naranja oscuro: énfasis sobre fondo claro
#let ink = rgb("#1B1E24")
#let paper = rgb("#FBFAF7")
#let muted = rgb("#6E7480")
#let soft = rgb("#F2EFE9")
#let hairline = rgb("#E4E0D7")
#let dark-bg = rgb("#14171D")
#let dark-fg = rgb("#EDEAE3")
#let dark-muted = rgb("#9AA0AB")
#let ok-green = rgb("#2EA043")
#let alert-red = rgb("#E5484D")

// ── Tipografía ───────────────────────────────────────────────────────────
// Space Grotesk (texto) y JetBrains Mono (código) viven en fonts/
// (descargadas por `make fonts`) y se cargan con --font-path fonts.
// Si faltan, se cae a las fuentes del sistema.
#let sans = ("Space Grotesk", "Helvetica Neue", "Arial")
#let mono = ("JetBrains Mono", "Menlo", "DejaVu Sans Mono")

// ── Reglas globales (aplicar en el punto de entrada con `#show: setup`) ──
#let setup(body) = {
  set document(
    title: "Si no lo mides, no lo controlas: Prometheus en 4 piezas",
    author: "Ken Esparta",
  )
  set page(
    paper: "presentation-16-9",
    margin: (x: 1.7cm, top: 1.4cm, bottom: 1.5cm),
    fill: paper,
  )
  set text(font: sans, size: 20pt, fill: ink, lang: "es", hyphenate: false)
  // Comillas inglesas (“ ”) en vez de las angulares («») que trae lang: "es".
  // Para volver a las angulares, borrar esta línea.
  set smartquote(quotes: (single: "‘’", double: "“”"))
  set par(leading: 0.62em, justify: false)
  set list(
    marker: box(baseline: -0.1em, square(size: 0.36em, fill: prom-orange)),
    indent: 2pt,
    body-indent: 0.65em,
    spacing: 1.15em,
  )

  show raw: set text(font: mono)
  show raw.where(block: false): it => box(
    fill: soft,
    stroke: 0.6pt + hairline,
    radius: 3.5pt,
    inset: (x: 5pt, y: 0pt),
    outset: (y: 3pt),
    text(size: 0.82em, it),
  )
  show raw.where(block: true): it => block(
    width: 100%,
    fill: soft,
    stroke: 1pt + hairline,
    radius: 9pt,
    inset: (x: 17pt, y: 14pt),
    text(size: 14pt, it),
  )

  // Space Grotesk no tiene cursiva: el énfasis (_..._) va en peso y color.
  show emph: it => text(style: "normal", weight: 500, fill: prom-ember, it.body)

  body
}

// ── Componentes ──────────────────────────────────────────────────────────
#let page-footer = context [
  #set text(size: 10pt, fill: rgb("#948F85"))
  Prometheus en 4 piezas
  #h(1fr)
  #counter(page).get().first() / #counter(page).final().first()
]

// Las 4 piezas como motivo gráfico: 4 cuadrados, el activo en naranja.
#let piezas-motif(active: 0, size: 11pt, on-dark: false) = {
  let off = if on-dark { rgb("#2C313B") } else { rgb("#E2DDD3") }
  stack(
    dir: ltr,
    spacing: 0.5 * size,
    ..range(4).map(i => rect(
      width: size,
      height: size,
      radius: 0.24 * size,
      fill: if active == 0 or active == i + 1 { prom-orange } else { off },
    )),
  )
}

#let divider = grid(
  columns: (46pt, 1fr),
  align: horizon,
  line(length: 100%, stroke: 3pt + prom-orange),
  line(length: 100%, stroke: 1pt + hairline),
)

#let callout(body) = block(
  width: 100%,
  fill: rgb("#FBEBE4"),
  stroke: (left: 3pt + prom-orange),
  inset: (x: 16pt, y: 11pt),
  text(size: 16.5pt, body),
)

#let badge(txt, color) = box(
  fill: color,
  radius: 4pt,
  inset: (x: 7pt, y: 2.5pt),
  baseline: 24%,
  text(size: 0.72em, weight: 700, fill: white, tracking: 0.04em, txt),
)

// Franja "DEMO": qué se abre en vivo (URL en localhost) y qué se ve.
#let demo-strip(body) = block(
  width: 100%,
  fill: dark-bg,
  radius: 10pt,
  inset: (x: 15pt, y: 11pt),
)[
  #show raw.where(block: false): it => text(
    font: mono,
    size: 0.92em,
    fill: rgb("#FFC5AE"),
    it.text,
  )
  #grid(
    columns: (auto, 1fr),
    column-gutter: 13pt,
    align: horizon,
    box(
      fill: prom-orange,
      radius: 4pt,
      inset: (x: 8pt, y: 4.5pt),
      text(size: 11pt, weight: 700, fill: white, tracking: 0.12em)[DEMO],
    ),
    text(size: 16pt, fill: dark-fg, body),
  )
]

// Diapositiva de contenido genérica.
#let slide(title, valign: top, body) = page(footer: page-footer)[
  #grid(
    rows: (auto, 1fr),
    row-gutter: 0pt,
    {
      text(size: 29pt, weight: 700, title)
      v(9pt)
      divider
      v(18pt)
    },
    align(valign, body),
  )
]

// Diapositiva de una de las 4 piezas (kicker + motivo + franja DEMO).
#let pieza(n, title, question, demo: none, body) = page(footer: page-footer)[
  #grid(
    rows: (auto, 1fr, auto),
    row-gutter: 0pt,
    {
      grid(
        columns: (1fr, auto),
        align: horizon,
        text(size: 12pt, weight: 700, tracking: 0.16em, fill: prom-orange)[PIEZA #n DE 4],
        piezas-motif(active: n),
      )
      v(12pt)
      grid(
        columns: (auto, 1fr),
        column-gutter: 16pt,
        align: bottom,
        text(size: 31pt, weight: 700, title),
        pad(bottom: 5pt, text(size: 18pt, fill: muted, question)),
      )
      v(9pt)
      divider
      v(18pt)
    },
    body,
    if demo == none { [] } else { pad(top: 14pt, demo) },
  )
]

// Portada y cierre sobre fondo oscuro, sin pie de página.
#let dark-slide(body) = page(
  fill: dark-bg,
  footer: none,
  margin: (x: 2.1cm, y: 1.9cm),
)[
  // Sin espaciado de párrafo: en estas portadas cada hueco lo pone un #v().
  #set par(spacing: 0pt)
  #set text(fill: dark-fg)
  #body
]

// El flujo de las 4 piezas como tarjetas encadenadas (intro y recap).
#let step-card(n, name, desc) = block(
  width: 100%,
  height: 118pt,
  fill: white,
  stroke: 1pt + hairline,
  radius: 11pt,
  inset: (x: 13pt, y: 12pt),
)[
  #text(size: 19pt, weight: 700, fill: prom-orange)[#n]
  #v(4pt)
  #text(size: 16.5pt, weight: 700, name)
  #v(3pt)
  #text(size: 12.5pt, fill: muted, desc)
]

#let pipeline(steps) = {
  let arrow = align(center, text(size: 17pt, weight: 700, fill: rgb("#B4AC9E"))[→])
  let cells = ()
  for (i, s) in steps.enumerate() {
    if i > 0 { cells.push(arrow) }
    cells.push(step-card(i + 1, s.first(), s.last()))
  }
  grid(
    columns: (1fr, auto, 1fr, auto, 1fr, auto, 1fr),
    column-gutter: 9pt,
    align: horizon,
    ..cells,
  )
}

// De lo simple a lo útil: el panel de queries de la pieza 3.
#let promql-panel(steps) = block(
  width: 100%,
  fill: soft,
  stroke: 1pt + hairline,
  radius: 10pt,
  inset: (x: 16pt, y: 15pt),
)[
  #grid(
    columns: (auto, 1fr, auto),
    column-gutter: 13pt,
    row-gutter: 15pt,
    align: horizon,
    ..steps
      .enumerate()
      .map(((i, s)) => (
        text(font: mono, size: 14pt, weight: 700, fill: prom-orange, str(i + 1)),
        text(font: mono, size: 13.5pt, s.first()),
        text(size: 12.5pt, fill: muted, s.last()),
      ))
      .flatten(),
  )
]

#let seguir-card(kicker, body) = block(
  width: 100%,
  height: 100%,
  fill: white,
  stroke: 1pt + hairline,
  radius: 11pt,
  inset: (x: 18pt, y: 16pt),
)[
  #text(size: 13pt, weight: 700, tracking: 0.1em, fill: prom-orange, kicker)
  #v(7pt)
  #text(size: 17.5pt, body)
]
