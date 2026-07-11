// 9 · Pieza 3 bis — las mismas 4 herramientas, ahora sobre las métricas de la app.
// Cierra el arco de PromQL: el lenguaje no cambia, cambia lo que preguntas.
#import "../theme.typ": *

#slide("Las mismas 4 herramientas, sobre la app")[
  #promql-panel(
    numerar: false,
    (
      ("sum(rate(pokeapi_http_peticiones_total[1m])) by (rol)", "tráfico por rol"),
      ("sum(rate(pokeapi_pokemon_consultas_total[5m])) by (origen)", "¿caché o API pública?"),
      ("pokeapi_sesiones_activas", "cuánta gente hay dentro"),
      ("sum(increase(pokeapi_login_errores_total[1m]))", "logins fallidos"),
    ),
  )
  #v(1fr)
  #callout[Cambia la app; el lenguaje, no. El _negocio_ se pregunta igual que el CPU.]
]
