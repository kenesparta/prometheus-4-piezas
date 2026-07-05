# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this repository is

This is primarily a **content repository**: it holds the speaker script ("guion") for a
Cloud Native conference talk titled *"Si no lo mides, no lo controlas: Prometheus en 4
piezas"* — a ~30 min introductory talk with live demos aimed at people with no prior
Prometheus experience. Alongside the prose there is a small set of Kubernetes manifests that
make the live demo runnable. There is no application source code, build system, or test suite.

- `guion-prometheus-4-piezas.md` — the narrative speaker script (what to *say*). Spanish.
- `DEMO.md` — the operational runbook (what to *type*): step-by-step demo, trucos, troubleshooting.
- `slides.md` — Marp slide deck aligned to the 4 pieces (`npx @marp-team/marp-cli slides.md -o slides.html`).
- `slides-typst/` — Typst version of the same deck (`make -C slides-typst`); mirrors
  `slides.md` slide for slide. One file per slide under `slides-typst/slides/`, shared look
  in `theme.typ`, vendored OFL fonts in `fonts/` (re-downloadable via `make fonts`). No
  Typst packages.
- `k8s/` — Kubernetes manifests for the demo + `README.md` (deploy guide) + `port-forward.sh`.
- `CLAUDE.md` — this file.

The three docs are deliberately separated: **guion = narration, DEMO.md = mechanics,
slides.md = visual anchors.** A change to the demo (a query, a port, an alert threshold)
usually needs to stay consistent across all three plus the manifests — and mirrored in
`slides-typst/slides.typ`, which duplicates the deck's content.

## Working language

All content is in **Spanish (es)** and should stay that way, following RAE/ASALE norms
(tildes, punctuation, `¿?`/`¡!`). When fixing spelling, accents, or punctuation, use the
`corregir-es` skill — it fixes mechanical errors only and must not change voice, structure,
or the technical decisions in the script.

## Structure of the talk (the "architecture")

The talk's whole pedagogy is built on one frame: **Prometheus reduced to 4 pieces**, taught
in order, each building on the previous and each with its own live demo. Preserve this
spine when editing — the transitions between sections explicitly hand off from one piece to
the next ("el cable que conecta...", "ahora que tenemos datos...").

1. **Exporters** — expose metrics over HTTP at `/metrics` as plain text (pull model, not push). Demo: `node_exporter` on `:9100`.
2. **El servidor** — scrapes each `/metrics` endpoint on an interval and stores time series; configured via `prometheus.yml`. Demo: Prometheus on `:9090`, `/targets` showing UP/DOWN.
3. **PromQL** — query language; taught simple→useful in 4 queries ending at CPU-usage %. The "ajá" moment; the script flags this as the section *not* to cut for time.
4. **Alertmanager** — an alert is just a PromQL query plus a threshold (`for`/`expr`); routes notifications. Demo: `/alerts` on `:9090`, optional Alertmanager on `:9093`.

The script also carries demo logistics that constrain edits: a pre-talk technical checklist,
a timing table per block, "what to point at on screen" notes, and time-cut guidance. Code
fences inside the script (`prometheus.yml`, `alert.rules.yml`, PromQL queries, shell
commands) are **demo artifacts shown to the audience** — keep them runnable and consistent
with the narration that references them (e.g. the alert `expr` is deliberately the same
query introduced in the PromQL section).

## How the demo is wired (k8s/)

The demo runs on a **Kubernetes cluster (GKE) provisioned outside this repo** (Terraform,
separate repo), accessed via `kubectl port-forward` so the guion's `localhost:9090/9100/9093`
URLs work unchanged. **Deployment happens via GitHub Actions from this repo**
(`.github/workflows/desplegar-k8s.yml` applies `k8s/` on push, authenticating to GCP via
Workload Identity Federation — no secrets, the IAM side lives in the cluster's Terraform
repo; `pokeapi-imagen.yml` builds the pokeapi image and pushes it to GitHub Packages,
`ghcr.io/kenesparta/pokeapi`, using the built-in `GITHUB_TOKEN` — the package must stay
public so GKE pulls it without an imagePullSecret) — do not add cluster-creation or deploy scripts
here; `kubectl apply -f k8s/` is the manual fallback. Deliberately plain manifests — no Operator, no `kube-prometheus-stack` — so each
of the 4 pieces stays visible:

- `node-exporter` (Deployment+Service, :9100) → Prometheus (Deployment+Service, :9090,
  config from a ConfigMap) → Alertmanager (Deployment+Service, :9093). `50-sample-app.yaml`
  is an optional bonus exporter.
- Prometheus uses **`static_configs`** (not `kubernetes_sd`) pointing at Service DNS
  (`node-exporter:9100`, `alertmanager:9093`) — keeps `prometheus.yml` close to the guion's.
- Two alert rules ship in the ConfigMap: `CPUAlto` (`> 80`, stays Inactive = healthy) and
  **`CPUAltoDemo` (`> 1`, fires in ~30s)**. The low-threshold rule exists so the live "watch
  it go red" moment needs no config reload — ConfigMap propagation to the pod takes ~1 min
  and would kill the demo timing. Don't "fix" `CPUAltoDemo` thinking it's a bug.
- Deploy 5–10 min before the talk: `rate(...[1m])` queries and the alert need scraped data.

Common entry points (full guide in `k8s/README.md`, runbook in `DEMO.md`):

```bash
kubectl apply -f k8s/                 # deploy all 4 pieces into namespace prometheus-demo
./k8s/port-forward.sh                 # expose 9090/9100/9093 on localhost (Ctrl-C to stop)
kubectl delete -f k8s/                # tear down
```

## Git notes

- The `origin` remote uses an SSH host alias: `gh:kenesparta/prometheus-4-piezas.git` (`gh:`
  is defined in the user's `~/.ssh/config`, not a standard host; it points at GitHub).
- The repo previously lived on Codeberg (Forgejo) with SHA-256 object format; it was
  re-created on GitHub with standard SHA-1 history. CI/CD is **GitHub Actions**
  (`.github/workflows/`), not Forgejo Actions — old docs or clones mentioning
  Codeberg/`.forgejo/` are stale.
