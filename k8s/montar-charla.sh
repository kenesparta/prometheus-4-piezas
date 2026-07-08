#!/usr/bin/env bash
# =============================================================================
# Montaje COMPLETO de la infra de la charla "Prometheus en 4 piezas", ANTES de
# empezar. Idempotente: se puede re-ejecutar sin miedo.
#
# Deja levantado, cada uno con su IP pública (LoadBalancer):
#   - pokeapi          → la app, para EL PÚBLICO
#   - prometheus       → la UI de las 4 piezas (/targets, PromQL, /alerts), TÚ
#   - grafana          → el dashboard bonito (acceso anónimo), TÚ
#
# Uso:
#   ./k8s/montar-charla.sh            # montar todo
#   ./k8s/montar-charla.sh --estado   # solo mostrar IPs/estado actual
#
# Requisitos previos (una sola vez):
#   - kubectl apuntando al cluster (gcloud container clusters get-credentials …)
#   - k8s/secrets.local.yaml con los Secrets pokeapi-redis y pokeapi-mongodb (gitignorado)
#   - La imagen ghcr.io/kenesparta/pokeapi debe ser PÚBLICA en GHCR
# =============================================================================
set -euo pipefail

NS="prometheus-demo"
CTX_ESPERADO="gke_kdc-lima_us-central1-c_kcd-main-cluster"
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"   # carpeta k8s/

# --- Espera a que un Service LoadBalancer reciba IP pública (hasta ~5 min) ----
espera_ip() {
  local svc="$1" ip=""
  for _ in $(seq 1 60); do
    ip="$(kubectl -n "$NS" get svc "$svc" \
      -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || true)"
    [[ -n "$ip" ]] && { echo "$ip"; return 0; }
    sleep 5
  done
  return 0   # devuelve vacío si no llegó a tiempo (no aborta el script)
}

# --- Resumen final con las URLs -----------------------------------------------
resumen() {
  local app prom graf
  app="$(espera_ip pokeapi-publico)"
  prom="$(espera_ip prometheus-publico)"
  graf="$(espera_ip grafana)"
  echo
  echo "  ┌─ URLs de la charla ─────────────────────────────────────────────"
  echo "  │  APP (público):   http://${app:-<IP pendiente>}"
  echo "  │  PROMETHEUS (tú): http://${prom:-<IP pendiente>}"
  echo "  │  GRAFANA (tú):    http://${graf:-<IP pendiente>}"
  echo "  │       acceso anónimo (solo ver) · admin / prometheus-charla"
  echo "  └─────────────────────────────────────────────────────────────────"
  echo
  if [[ -n "${app:-}" ]] && curl -fsS -m 5 "http://$app/salud" >/dev/null 2>&1; then
    echo "  ✅ la app responde en /salud"
  else
    echo "  ⚠️  la app aún no responde (dale 1-2 min tras asignarse la IP)"
  fi
  echo "  Para bajar todo tras la charla:  kubectl delete namespace $NS"
}

# --- Verificación de contexto (evita desplegar en el cluster equivocado) ------
CTX="$(kubectl config current-context)"
echo "==> Contexto kubectl: $CTX"
if [[ "$CTX" != "$CTX_ESPERADO" ]]; then
  echo "!!  No es el cluster esperado ($CTX_ESPERADO)."
  echo "    Arréglalo con:"
  echo "      gcloud container clusters get-credentials kcd-main-cluster \\"
  echo "        --zone us-central1-c --project kdc-lima"
  exit 1
fi

# --- Modo "solo estado" -------------------------------------------------------
if [[ "${1:-}" == "--estado" ]]; then
  kubectl -n "$NS" get deploy,svc,pods 2>/dev/null || echo "(namespace aún sin desplegar)"
  resumen
  exit 0
fi

# --- 1) Namespace + Secrets Redis/Mongo (deben ir ANTES que pokeapi) ----------
echo "==> 1/4  Namespace y Secrets (Redis + Mongo)"
kubectl apply -f "$DIR/00-namespace.yaml"
if [[ -f "$DIR/secrets.local.yaml" ]]; then
  kubectl apply -f "$DIR/secrets.local.yaml"
else
  echo "!!  Falta $DIR/secrets.local.yaml (Secrets pokeapi-redis y pokeapi-mongodb)."
  echo "    Créalos con tus URLs reales, p. ej.:"
  echo "      kubectl create secret generic pokeapi-redis -n $NS \\"
  echo "        --from-literal=REDIS_URL='rediss://usuario:PASS@host:puerto'"
  echo "      kubectl create secret generic pokeapi-mongodb -n $NS \\"
  echo "        --from-literal=MONGODB_URI='mongodb+srv://usuario:PASS@host/...'"
  exit 1
fi

# --- 2) Las 4 piezas + pokeapi + Grafana --------------------------------------
echo "==> 2/4  Aplicando manifiestos (node-exporter, prometheus, alertmanager, grafana, pokeapi)"
kubectl apply -f "$DIR/"

# --- 3) Esperar a que los pods estén listos -----------------------------------
echo "==> 3/4  Esperando a que los Deployments estén disponibles (hasta 3 min)"
kubectl -n "$NS" wait --for=condition=Available --timeout=180s deploy --all || true
echo "--- pods ---"
kubectl -n "$NS" get pods

# --- 4) IPs públicas + resumen ------------------------------------------------
echo "==> 4/4  Esperando las IPs públicas (LoadBalancer, ~1-3 min cada una)"
resumen
