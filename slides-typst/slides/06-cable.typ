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
```

  #v(1fr)
  #callout["Ve a `node-exporter:9100` cada 15 s." Eso es todo.]
]
