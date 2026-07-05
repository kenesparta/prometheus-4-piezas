// 8 · Pieza 4 — Alertmanager (el expr es la misma query de la pieza 3)
#import "../theme.typ": *

#pieza(
  4,
  "Alertmanager",
  "Que el sistema te avise",
  demo: demo-strip[`localhost:9090/alerts` #h(8pt) → #h(8pt) #badge("FIRING", alert-red) #h(8pt) → #h(8pt) llega a `localhost:9093`],
)[
  - Una *regla de alerta* = una query de PromQL + un umbral + `for`.
  - *Alertmanager* recibe, agrupa, silencia y enruta (Slack, email, PagerDuty).

  #v(6pt)
```yaml
- alert: CPUAlto
  expr: 100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[1m])) * 100) > 80
  for: 1m
```
  #v(10pt)
  #callout[El `expr` es la misma query de la Pieza 3. Nada nuevo.]
]
