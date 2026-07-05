# Manifiestos de la demo — Prometheus en 4 piezas (GKE)

Despliegue mínimo y explícito de las 4 piezas en un cluster de Kubernetes
(pensado para un cluster GKE ya montado). Sin Operator ni `kube-prometheus-stack`:
cada pieza es un manifiesto legible.

| Archivo | Pieza | Qué crea |
|---|---|---|
| `00-namespace.yaml` | — | Namespace `prometheus-demo` |
| `10-node-exporter.yaml` | 1 · Exporter | `node-exporter` (Deployment + Service, :9100) |
| `20-prometheus-config.yaml` | 2 + 4 | ConfigMap: `prometheus.yml` + `alert.rules.yml` |
| `22-prometheus.yaml` | 2 · Servidor | `prometheus` (Deployment + Service, :9090) |
| `30-alertmanager.yaml` | 4 · Alertmanager | `alertmanager` (ConfigMap + Deployment + Service, :9093) |
| `50-sample-app.yaml` | 1 (bonus, opcional) | `app-ejemplo` instrumentada (:8080) |
| `60-pokeapi.yaml` | 1–4 (demo interactiva) | `pokeapi` (Deployment + Service :3000 + LoadBalancer público) |

> `60-pokeapi.yaml` necesita la imagen publicada y el Secret `pokeapi-redis`
> antes de aplicarse (paso a paso en `pokeapi/USO-IMAGEN.md`). El resto de la
> demo funciona igual sin él.

## Requisitos

- `kubectl` configurado contra tu cluster. Si es GKE:
  ```bash
  gcloud container clusters get-credentials <CLUSTER> --region <REGION> --project <PROJECT>
  ```
- Verifica el contexto antes de aplicar nada:
  ```bash
  kubectl config current-context
  ```

## Desplegar

El despliegue lo hace el **CI/CD del repositorio (GitHub Actions)**: el workflow
`.github/workflows/desplegar-k8s.yml` aplica estos manifiestos contra el cluster
(aprovisionado aparte, fuera de este repo) en cada push a `main` que toque `k8s/`;
aquí no hay scripts de despliegue. La autenticación va por **Workload Identity
Federation** (token OIDC de GitHub, sin secrets ni llaves): el pool y el rol
`roles/container.developer` están declarados en el repo de Terraform del cluster
(`kcd-lima-k8s-kong/terraform/github-actions.tf`). La
imagen de pokeapi la construye y publica en GitHub Packages
(`ghcr.io/kenesparta/pokeapi`) el workflow `.github/workflows/pokeapi-imagen.yml`;
el package debe ser público para que el cluster la baje sin `imagePullSecret`.

A mano, si hace falta (con `kubectl` ya apuntando al cluster):

```bash
kubectl apply -f k8s/
# (el namespace 00 se crea primero por orden alfabético)

# Secrets que pokeapi espera (las credenciales reales viven en
# k8s/secrets.local.yaml, gitignorado — nunca se commitean):
kubectl apply -f k8s/secrets.local.yaml

kubectl -n prometheus-demo get pods -w   # esperar a Running/Ready
```

> Despliega **5–10 minutos antes** de la charla: las queries con `rate(...[1m])`
> y la alerta `CPUAltoDemo` necesitan un par de minutos de datos scrapeados para
> dar valores estables.

## Acceder (port-forward)

Las URLs del guion (`localhost:9090/9100/9093`) se sirven vía port-forward:

```bash
./k8s/port-forward.sh        # abre los 3 a la vez; Ctrl-C cierra todos
```

O manualmente, una terminal por servicio:

```bash
kubectl -n prometheus-demo port-forward svc/prometheus    9090:9090
kubectl -n prometheus-demo port-forward svc/node-exporter 9100:9100
kubectl -n prometheus-demo port-forward svc/alertmanager  9093:9093
```

## Recargar config sin reiniciar

El pod de Prometheus arranca con `--web.enable-lifecycle`:

```bash
kubectl -n prometheus-demo apply -f k8s/20-prometheus-config.yaml
# la propagación del ConfigMap al pod tarda ~1 min; luego:
curl -X POST http://localhost:9090/-/reload
```

## Limpieza

```bash
kubectl delete -f k8s/         # o: kubectl delete namespace prometheus-demo
```

## Versiones de imágenes

`prometheus v3.0.0` · `node-exporter v1.8.2` · `alertmanager v0.27.0` ·
`prometheus-example-app v0.5.0`. Si alguna imagen no baja en tu cluster, ajusta
el tag al último estable de [prometheus.io/download](https://prometheus.io/download/).
