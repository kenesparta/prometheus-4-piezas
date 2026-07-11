// 6 · El cable (prometheus.yml) — la config que conecta las piezas 1 y 2
#import "../theme.typ": *

#slide("El cable (prometheus.yml)")[
```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'node'
    static_configs:
      - targets: ['node-exporter:9100']

  - job_name: 'pokeapi'          # nuestra app, un target más
    static_configs:
      - targets: ['pokeapi:3000']
```

  #v(1fr)
  #callout["Ve a estas direcciones cada 15 s." Eso es todo.]
]
