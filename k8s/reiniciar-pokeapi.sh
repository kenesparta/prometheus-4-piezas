#!/usr/bin/env bash
# =============================================================================
# Reinicia el servicio pokeapi para que RELEA el Secret (REDIS_URL / MONGODB_URI).
#
# ¿Cuándo usarlo?
#   Cuando el pod se queda sin comunicar con Redis/Mongo AUNQUE la credencial sea
#   correcta — típico tras probar una caída (dejar caer Redis) o tras tocar el
#   Secret. Motivo: Kubernetes inyecta las variables de los Secrets como env
#   vars al ARRANCAR el pod y NO las recarga en caliente. El ConnectionManager
#   reconecta solo ante un blip de red con la MISMA credencial, pero un cambio
#   de credencial/Secret exige reiniciar el pod para que vuelva a leer la env.
#
#   Síntoma claro en los logs del pod:
#     WARN ... error de infraestructura: Password authentication failed
#   mientras `redis-cli -u 'redis://...'` sí responde PONG.
#
# Qué hace: NO es un rollout ni redespliega imagen. Solo borra el pod; el
# Deployment lo recrea al instante, releyendo el Secret actual.
#
# Uso:
#   ./k8s/reiniciar-pokeapi.sh            # reinicia el pod y verifica /salud
#   ./k8s/reiniciar-pokeapi.sh --secret   # además re-aplica k8s/secrets.local.yaml
#                                         # (por si el Secret del cluster quedó mal)
# =============================================================================
set -euo pipefail

NS="prometheus-demo"
CTX_ESPERADO="gke_kdc-lima_us-central1-c_kcd-main-cluster"
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"   # carpeta k8s/

# --- Verificación de contexto (no reiniciar en el cluster equivocado) ---------
CTX="$(kubectl config current-context 2>/dev/null || true)"
echo "==> Contexto kubectl: ${CTX:-<ninguno>}"
if [[ "$CTX" != "$CTX_ESPERADO" ]]; then
  echo "!!  No es el cluster esperado ($CTX_ESPERADO). Arréglalo con:"
  echo "      gcloud container clusters get-credentials kcd-main-cluster \\"
  echo "        --zone us-central1-c --project kdc-lima"
  exit 1
fi

# --- Comprobar que la demo está montada ---------------------------------------
if ! kubectl -n "$NS" get deploy pokeapi >/dev/null 2>&1; then
  echo "!!  No existe el Deployment 'pokeapi' en el namespace $NS."
  echo "    ¿Montaste la demo?  ./k8s/montar-charla.sh"
  exit 1
fi

# --- Opcional: re-aplicar el Secret -------------------------------------------
if [[ "${1:-}" == "--secret" ]]; then
  if [[ -f "$DIR/secrets.local.yaml" ]]; then
    echo "==> Re-aplicando Secrets (Redis + Mongo) desde secrets.local.yaml"
    kubectl apply -f "$DIR/secrets.local.yaml"
  else
    echo "!!  Falta $DIR/secrets.local.yaml (Secrets pokeapi-redis y pokeapi-mongodb)."
    exit 1
  fi
fi

# --- Reinicio: borrar el pod → el Deployment lo recrea (relee el Secret) -------
echo "==> Reiniciando el servicio pokeapi (borro el pod; el Deployment lo recrea)"
kubectl -n "$NS" delete pod -l app=pokeapi

echo "==> Esperando a que el pod nuevo esté Ready (hasta 2 min)"
kubectl -n "$NS" rollout status deploy/pokeapi --timeout=120s

# --- Verificación: /salud vía port-forward (no depende de la IP pública) -------
echo "==> Verificando /salud (port-forward temporal a :13000)"
kubectl -n "$NS" port-forward svc/pokeapi 13000:3000 >/dev/null 2>&1 &
PF=$!
trap 'kill "$PF" 2>/dev/null || true' EXIT

SALUD=""
for _ in 1 2 3 4 5 6; do
  sleep 2
  SALUD="$(curl -s -m 5 http://127.0.0.1:13000/salud 2>/dev/null || true)"
  [[ -n "$SALUD" ]] && break
done
echo "    /salud -> ${SALUD:-<sin respuesta>}"

# --- Diagnóstico final --------------------------------------------------------
if echo "$SALUD" | grep -q '"redis":"ok"' && echo "$SALUD" | grep -q '"mongo":"ok"'; then
  echo "✅ Reconectado: Redis y Mongo OK."
elif echo "$SALUD" | grep -q '"redis":"ok"'; then
  echo "✅ Redis OK. ⚠️  Mongo aún no; revisa MONGODB_URI / que Atlas esté despierto."
else
  echo "⚠️  Redis sigue sin conectar tras el reinicio."
  echo "    Si en los logs ves 'Password authentication failed', el Secret del CLUSTER"
  echo "    tiene una credencial que no autentica (no solo el pod). Corrige la URL en"
  echo "    k8s/secrets.local.yaml y re-lanza:  ./k8s/reiniciar-pokeapi.sh --secret"
  echo "    Logs recientes:"
  kubectl -n "$NS" logs -l app=pokeapi --tail=5 2>/dev/null | grep -iE "redis|mongo|error|auth" || true
fi
