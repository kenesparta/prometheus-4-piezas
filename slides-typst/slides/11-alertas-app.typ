// 11 · Pieza 4 bis — las reglas de alerta de la app. El remate: hasta una
// alerta de seguridad es una query de PromQL + un umbral + un `for`.
// El expr va con escalar plegado (>-) de YAML para que quepa en la línea.
#import "../theme.typ": *

#slide("Las alertas de la app")[
  #alertas-tabla((
    ("PokeapiCaida", [la app no responde al scrape (target DOWN)]),
    ("PokeapiSinRedis", [perdió el caché y las sesiones]),
    ("PokeapiSinMongo", [perdió la base de usuarios]),
    ("PokeapiFuerzaBrutaDemo", [demasiados passwords incorrectos por minuto]),
  ))

  #v(16pt)
```yaml
- alert: PokeapiFuerzaBrutaDemo
  expr: >-
    sum(increase(pokeapi_login_errores_total{motivo="password_incorrecto"}[1m]))
    > 5
  for: 30s
```
  #v(1fr)
  #callout[Hasta una alerta de _seguridad_ es lo mismo: una pregunta, un umbral y un `for`.]
]
