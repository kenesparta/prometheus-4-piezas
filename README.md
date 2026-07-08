# Si no lo mides, no lo controlas: Prometheus en 4 piezas

Materiales de una **ponencia Cloud Native** (~30 min, con demo en vivo) que
desmitifica Prometheus para gente **sin experiencia previa**: lo reduce a sus
**4 piezas esenciales** y demuestra que con esos cuatro conceptos ya entiendes
el 80 % de cómo funciona. Pasamos de un sistema "ciego" a uno que se mide, se
consulta y se alerta a sí mismo.

> Prometheus es el **segundo proyecto graduado** de la CNCF (después de
> Kubernetes) y el punto de partida de casi cualquier stack Cloud Native.

Este es, sobre todo, un **repositorio de contenido**: el guion de la charla y el
material para reproducir las demos. No hay un "producto" que compilar —salvo la
app interactiva `pokeapi/`, que es en sí misma una de las demos.

## Las 4 piezas

La pedagogía entera de la charla se apoya en una idea: **Prometheus son 4
piezas**, enseñadas en orden y cada una con su demo en vivo.

| # | Pieza | En una frase | Demo |
|---|---|---|---|
| 1 | **Exporters** | Exponen métricas por HTTP en `/metrics`, texto plano (modelo *pull*, no *push*). | `node_exporter` en `:9100` |
| 2 | **El servidor** | Hace *scrape* de cada `/metrics` en un intervalo y guarda las series temporales; se configura con `prometheus.yml`. | Prometheus en `:9090`, `/targets` |
| 3 | **PromQL** | El lenguaje para preguntarles cosas a esas métricas. Es el momento "ajá". | Consultas simples → uso de CPU en % |
| 4 | **Alertmanager** | Una alerta es solo una consulta PromQL + un umbral (`expr`/`for`); enruta los avisos. | `/alerts` en `:9090`, Alertmanager en `:9093` |

## Qué hay en este repo

Tres documentos, tres papeles distintos: **guion = narración**, **DEMO.md =
mecánica**, **diapositivas = anclas visuales**.

| Ruta | Qué es |
|---|---|
| [`guion-prometheus-4-piezas.md`](guion-prometheus-4-piezas.md) | El **guion narrativo**: qué *decir*. Resumen, checklist previo, tabla de tiempos, notas de "qué señalar en pantalla" y guía de recortes por tiempo. |
| [`DEMO.md`](DEMO.md) | El **runbook operativo**: qué *teclear*, paso a paso, trucos y troubleshooting. |
| [`slides.md`](slides.md) | Diapositivas en **Marp** (anclas visuales, no para leer). |
| [`slides-typst/`](slides-typst/README.md) | Las mismas diapositivas en **Typst**, espejo de `slides.md`. |
| [`k8s/`](k8s/README.md) | **Manifiestos de Kubernetes** para la demo + `port-forward.sh` + `montar-charla.sh`. |
| [`pokeapi/`](pokeapi/README.md) | **App interactiva** (Rust + Leptos + Redis) que el público usa en vivo mientras las métricas aparecen en Prometheus. |
| [`.github/workflows/`](.github/workflows) | CI/CD (GitHub Actions): construye la imagen de `pokeapi` y despliega `k8s/` a GKE. |
| [`CLAUDE.md`](CLAUDE.md) | Guía para agentes de IA (Claude Code) al trabajar en el repo. |

## La demo en 5 minutos

Corre sobre un **cluster de Kubernetes (GKE)** aprovisionado aparte (Terraform,
en otro repo). Se accede vía `kubectl port-forward`, así que las URLs del guion
(`localhost:9090/9100/9093`) funcionan sin cambios.

```bash
kubectl apply -f k8s/     # despliega las 4 piezas en el namespace prometheus-demo
./k8s/port-forward.sh     # expone 9090/9100/9093 en localhost (Ctrl-C para cerrar)
kubectl delete -f k8s/    # limpieza al terminar
```

> **Despliega 5–10 minutos antes** de la charla: las consultas con
> `rate(...[1m])` y la alerta de demo necesitan un par de minutos de datos
> scrapeados. Guía completa en [`k8s/README.md`](k8s/README.md); mecánica de la
> demo en [`DEMO.md`](DEMO.md).

Los manifiestos son **deliberadamente planos** —sin Operator ni
`kube-prometheus-stack`— para que cada una de las 4 piezas siga siendo visible.
Prometheus usa `static_configs` apuntando al DNS de los Services, no
`kubernetes_sd`, para que `prometheus.yml` se parezca al del guion. Grafana va
como complemento opcional (dashboard "bonito"); las 4 piezas se enseñan en la UI
nativa de Prometheus.

## La app interactiva: `pokeapi/`

Una pokédex web en **Rust + Leptos (SSR + hidratación) con Redis**, en workspace
DDD/hexagonal. El público la usa en vivo (login, búsqueda de pokémon con caché,
roles) y cada acción genera tráfico que aparece en Prometheus: la propia app
expone `/metrics` (pieza 1), Prometheus la scrapea (pieza 2), se consulta con
PromQL (pieza 3) y tiene sus propias alertas (pieza 4). Detalles y cómo correrla
en [`pokeapi/README.md`](pokeapi/README.md); usar la imagen ya publicada, en
[`pokeapi/USO-IMAGEN.md`](pokeapi/USO-IMAGEN.md).

## Diapositivas

Dos versiones del mismo deck, espejo una de la otra:

```bash
npx @marp-team/marp-cli slides.md -o slides.html   # Marp → HTML (o --pdf)
make -C slides-typst                                # Typst → slides-typst/slides.pdf
```

## Mantener todo consistente

Una consulta, un puerto o un umbral de alerta viven **repetidos a propósito** en
varios sitios. Cualquier cambio a un detalle de la demo debe quedar consistente
en: el **guion**, **`DEMO.md`**, **`slides.md`**, las diapositivas de
**`slides-typst/`** y los **manifiestos de `k8s/`**. (Más contexto para editar
sin romper nada en [`CLAUDE.md`](CLAUDE.md).)

El texto está en **español** y se mantiene así, siguiendo la norma RAE/ASALE.

## Despliegue y CI/CD

El despliegue lo hace **GitHub Actions**, no scripts en este repo:

- `desplegar-k8s.yml` aplica `k8s/` contra el cluster en cada push a `main` que
  toque esos manifiestos. Autentica con **Workload Identity Federation** (OIDC,
  **sin secrets ni llaves**); el lado IAM vive en el Terraform del cluster.
- `pokeapi-imagen.yml` construye la imagen de `pokeapi` y la publica en GitHub
  Packages (`ghcr.io/kenesparta/pokeapi`) con el `GITHUB_TOKEN` integrado. El
  package debe mantenerse **público** para que GKE la baje sin `imagePullSecret`.

`kubectl apply -f k8s/` a mano es el *fallback* manual.
