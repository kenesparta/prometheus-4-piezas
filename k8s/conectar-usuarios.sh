#!/usr/bin/env bash
# =============================================================================
# Sube el volumen de "usuarios conectados" de la demo: crea sesiones en la app
# para que el gauge pokeapi_sesiones_activas ("esos son ustedes", Acto 3) suba.
#
# Mecánica (verificada en el código): cada POST /api/login con credenciales
# válidas mina un TOKEN nuevo -> pokeapi:sesion:{token} (TTL) + zadd al ZSET
# pokeapi:sesiones. El gauge = ZCARD de ese ZSET. Por eso N logins = N sesiones
# vivas (aunque sean del mismo usuario). TTL por defecto 24 h: se acumulan.
#
# Uso:
#   ./k8s/conectar-usuarios.sh                    # 50 sesiones (admin/123)
#   ./k8s/conectar-usuarios.sh --usuarios 200     # sube el número
#   ./k8s/conectar-usuarios.sh --registrar        # crea usuarios VISITOR nuevos
#                                                  #  (registro en Mongo) y los loguea
#   ./k8s/conectar-usuarios.sh --target http://34.55.181.146
#
# --registrar además mueve pokeapi_usuarios_registrados_total y
# pokeapi_usuarios_por_rol{rol="VISITOR"} (por eso escribe en MongoDB).
#
# Destino: por defecto la IP pública de la app (Service pokeapi-publico); si no,
# http://localhost:3000. Override con --target o TARGET=...
# =============================================================================
set -euo pipefail

NS="prometheus-demo"
PROM_IP_DEFAULT="34.10.208.11"   # solo para el aviso final; no crítico

TARGET="${TARGET:-}"
USUARIOS=50
WORKERS=10
TIMEOUT=8
REGISTRAR=0
USUARIO="admin"
PASSWORD="123"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --usuarios)  USUARIOS="$2"; shift 2;;
    --workers)   WORKERS="$2"; shift 2;;
    --target)    TARGET="$2"; shift 2;;
    --usuario)   USUARIO="$2"; shift 2;;
    --password)  PASSWORD="$2"; shift 2;;
    --registrar) REGISTRAR=1; shift;;
    -h|--help)   sed -n '2,30p' "$0" | sed 's/^# \{0,1\}//'; exit 0;;
    *) echo "Opción desconocida: $1  (usa -h)"; exit 1;;
  esac
done

# --- Destino -----------------------------------------------------------------
if [[ -z "$TARGET" ]]; then
  ip="$(kubectl -n "$NS" get svc pokeapi-publico \
        -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || true)"
  if [[ -n "$ip" ]]; then TARGET="http://$ip"; else TARGET="http://localhost:3000"; fi
fi
TARGET="${TARGET%/}"

command -v curl >/dev/null || { echo "!! falta curl"; exit 1; }
if ! curl -fsS -m "$TIMEOUT" "$TARGET/vivo" >/dev/null 2>&1; then
  echo "!! $TARGET no responde en /vivo. ¿App montada? ¿IP correcta?"; exit 1
fi

# --- Preflight de credenciales (modo por defecto) ----------------------------
if [[ "$REGISTRAR" == 0 ]]; then
  code=$(curl -s -o /dev/null -w '%{http_code}' -m "$TIMEOUT" -X POST "$TARGET/api/login" \
    -H 'content-type: application/json' -d "{\"nombre\":\"$USUARIO\",\"password\":\"$PASSWORD\"}")
  if [[ "$code" != "200" ]]; then
    echo "!! Login de prueba con $USUARIO devolvió HTTP $code (esperaba 200)."
    echo "   ¿Cambió la contraseña? ¿Mongo sembró el admin? (curl $TARGET/salud)"
    echo "   Usa --usuario/--password, o --registrar para crear usuarios nuevos."
    exit 1
  fi
fi

echo "┌─ conectar-usuarios ───────────────────────────────────────"
echo "│  destino:   $TARGET"
echo "│  crear:     $USUARIOS sesiones    workers: $WORKERS"
if [[ "$REGISTRAR" == 1 ]]; then
  echo "│  modo:      registrar usuarios VISITOR nuevos (registro en Mongo) + login"
else
  echo "│  modo:      login repetido de '$USUARIO' (sesiones en Redis, sin tocar Mongo)"
fi
echo "│  → sube:    pokeapi_sesiones_activas  (Acto 3: 'esos son ustedes')"
echo "└───────────────────────────────────────────────────────────"

WORK="$(mktemp -d)"
PIDS=()
trap 'rm -rf "$WORK" 2>/dev/null || true' EXIT

# --- Cuotas por worker (sin locks: suman exactamente USUARIOS) ----------------
base=$((USUARIOS / WORKERS)); extra=$((USUARIOS % WORKERS))

worker() {
  set +e
  local id="$1" quota="$2" f="$WORK/w$1" ok=0 err=0 code
  for ((k = 0; k < quota; k++)); do
    [[ -e "$WORK/stop" ]] && break
    if [[ "$REGISTRAR" == 1 ]]; then
      local u="demo-$id-$k-$RANDOM"
      curl -s -o /dev/null -m "$TIMEOUT" -X POST "$TARGET/api/registro" \
        -H 'content-type: application/json' \
        -d "{\"nombre\":\"$u\",\"password\":\"clave-demo-1234\"}"
      code=$(curl -s -o /dev/null -w '%{http_code}' -m "$TIMEOUT" -X POST "$TARGET/api/login" \
        -H 'content-type: application/json' \
        -d "{\"nombre\":\"$u\",\"password\":\"clave-demo-1234\"}")
    else
      code=$(curl -s -o /dev/null -w '%{http_code}' -m "$TIMEOUT" -X POST "$TARGET/api/login" \
        -H 'content-type: application/json' \
        -d "{\"nombre\":\"$USUARIO\",\"password\":\"$PASSWORD\"}")
    fi
    if [[ "$code" == "200" || "$code" == "201" ]]; then ok=$((ok + 1)); else err=$((err + 1)); fi
    printf '%s %s\n' "$ok" "$err" > "$f"
  done
  printf '%s %s\n' "$ok" "$err" > "$f"
}

for i in $(seq 1 "$WORKERS"); do
  quota=$base; [[ "$i" -le "$extra" ]] && quota=$((base + 1))
  [[ "$quota" -eq 0 ]] && continue
  worker "$i" "$quota" &
  PIDS+=("$!")
done

# --- Progreso en vivo --------------------------------------------------------
suma() { cat "$WORK"/w* 2>/dev/null | awk '{o+=$1;e+=$2} END{printf "%d %d", o, e}'; }
PARAR=0
trap 'PARAR=1' INT TERM
while :; do
  sleep 1 || true
  read -r ok err <<<"$(suma)"; ok=${ok:-0}; err=${err:-0}
  printf '\r  creadas %s/%s  (fallos %s)   ' "$ok" "$USUARIOS" "$err"
  [[ "$PARAR" == 1 ]] && break
  [[ $((ok + err)) -ge "$USUARIOS" ]] && break
done
[[ "$PARAR" == 1 ]] && touch "$WORK/stop"
wait 2>/dev/null || true
read -r ok err <<<"$(suma)"; ok=${ok:-0}; err=${err:-0}
echo
echo "└─ listo: $ok sesiones creadas ($err fallos)."

# --- Comprobación: valor del gauge en Prometheus -----------------------------
prom="http://$PROM_IP_DEFAULT"
val=$(curl -fsS -m 8 "$prom/api/v1/query" \
  --data-urlencode 'query=pokeapi_sesiones_activas' 2>/dev/null \
  | grep -oE '"[0-9.]+"\]' | tail -1 | tr -d '"]' || true)
if [[ -n "${val:-}" ]]; then
  echo "   pokeapi_sesiones_activas ahora ≈ $val   (refresca cada ~15 s tras el scrape)"
else
  echo "   Míralo en PromQL:  pokeapi_sesiones_activas"
fi
echo "   (Las sesiones viven con TTL ~24 h; se van solas o con: kubectl delete ns $NS)"
