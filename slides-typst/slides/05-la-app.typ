// 5 · La app de la demo: pokeapi — la pieza 1 aplicada a una app de verdad.
// Va justo después de Exporters: la app *es* un exporter, hecho en casa.
#import "../theme.typ": *

#slide("La app que vamos a observar")[
  #grid(
    columns: (1.25fr, 1fr),
    column-gutter: 28pt,
    {
      text(size: 18pt)[*pokeapi* — una pokédex web en Rust + Leptos. La van a usar ustedes.]
      v(12pt)
      text(size: 16.5pt)[
        - Buscas un pokémon: la 1.ª vez sale de la PokeAPI pública; las siguientes, del _caché en Redis_ — y se nota en la latencia.
        - Login, registro y roles (`ADMIN` / `EDITOR` / `VISITOR`).
        - Ella misma expone `/metrics`: un _exporter hecho en casa_.
      ]
    },
    stack(
      spacing: 11pt,
      dep-card("MongoDB", "los usuarios"),
      dep-card("Redis", "sesiones y caché de fichas"),
      dep-card("PokeAPI pública", "el origen de los datos"),
    ),
  )
  #v(1fr)
  #callout[No es un exporter de juguete: es _tu_ app instrumentada. Eso es lo que harás en el trabajo.]
]
