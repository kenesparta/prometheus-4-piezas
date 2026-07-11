#!/usr/bin/env bash
# =============================================================================
# Generador de carga para la demo "Prometheus en 4 piezas". Sube las req/sec
# contra la app (pokeapi) para que las curvas de PromQL se muevan y salten las
# alertas *Demo. Versión concurrente y con caudal regulable de los bucles
# sueltos de DEMO.md ("Trucos").
#
# Qué enciende:
#   - tráfico (por defecto): GET /api/pokemon/<n>  -> pokeapi_http_peticiones_total
#                            => alerta PokeapiTraficoAltoDemo (rate > 0.5)
#   - --fuerza-bruta:        POST /api/login (admin + password mala, sostenido)
#                            => pokeapi_login_errores_total{motivo=password_incorrecto}
#                            => alerta PokeapiFuerzaBrutaDemo (>5 en 1 min)
#   - --mixto:               ambas (mayoría tráfico + una fracción de logins malos)
#
# Uso:
#   ./k8s/generar-carga.sh                       # tráfico, ~25 req/s, 120 s
#   ./k8s/generar-carga.sh --rps 80              # sube el caudal
#   ./k8s/generar-carga.sh --rps 0               # a tope (sin freno)
#   ./k8s/generar-carga.sh --fuerza-bruta        # dispara la alerta de brute force
#   ./k8s/generar-carga.sh --duracion 0          # hasta Ctrl-C
#   ./k8s/generar-carga.sh --target http://34.55.181.146
#
# Destino: por defecto la IP pública de la app (Service pokeapi-publico); si no
# la encuentra, cae a http://localhost:3000 (port-forward). Overridable con
# --target o TARGET=...
# =============================================================================
set -euo pipefail

NS="prometheus-demo"

# --- Parámetros por defecto --------------------------------------------------
TARGET="${TARGET:-}"
RPS=25             # req/seg objetivo agregado (0 = sin freno, a tope)
WORKERS=12         # procesos concurrentes de curl
DURACION=120       # segundos (0 = hasta Ctrl-C)
TIMEOUT=5          # -m de curl
MODO="trafico"     # trafico | fuerza-bruta | mixto

while [[ $# -gt 0 ]]; do
  case "$1" in
    --target)        TARGET="$2"; shift 2;;
    --rps)           RPS="$2"; shift 2;;
    --workers)       WORKERS="$2"; shift 2;;
    --duracion)      DURACION="$2"; shift 2;;
    --fuerza-bruta)  MODO="fuerza-bruta"; shift;;
    --mixto)         MODO="mixto"; shift;;
    -h|--help)       sed -n '2,33p' "$0" | sed 's/^# \{0,1\}//'; exit 0;;
    *) echo "Opción desconocida: $1  (usa -h)"; exit 1;;
  esac
done

# --- Destino: --target > TARGET > IP pública > localhost ---------------------
if [[ -z "$TARGET" ]]; then
  ip="$(kubectl -n "$NS" get svc pokeapi-publico \
        -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || true)"
  if [[ -n "$ip" ]]; then TARGET="http://$ip"; else TARGET="http://localhost:3000"; fi
fi
TARGET="${TARGET%/}"   # sin barra final

# --- Preflight: ¿responde? ---------------------------------------------------
command -v curl >/dev/null || { echo "!! falta curl"; exit 1; }
if ! curl -fsS -m "$TIMEOUT" "$TARGET/vivo" >/dev/null 2>&1; then
  echo "!! $TARGET no responde en /vivo."
  echo "   ¿Está montada la app? ¿La IP correcta? (kubectl -n $NS get svc pokeapi-publico)"
  echo "   O usa el port-forward: ./k8s/port-forward.sh  y  --target http://localhost:3000"
  exit 1
fi

# --- Freno por worker para aproximar el RPS ----------------------------------
if [[ "$RPS" == 0 ]]; then
  DELAY=0
else
  DELAY=$(awk -v w="$WORKERS" -v r="$RPS" 'BEGIN{printf "%.3f", w/r}')
fi

echo "┌─ generar-carga ───────────────────────────────────────────"
echo "│  destino:   $TARGET"
echo "│  modo:      $MODO"
echo "│  workers:   $WORKERS    rps objetivo: $([[ "$RPS" == 0 ]] && echo 'a tope' || echo "$RPS")    freno/worker: ${DELAY}s"
echo "│  duración:  $([[ "$DURACION" == 0 ]] && echo 'hasta Ctrl-C' || echo "${DURACION}s")"
case "$MODO" in
  trafico)      echo "│  → mira:    PokeapiTraficoAltoDemo (Firing en ~30-60 s)";;
  fuerza-bruta) echo "│  → mira:    PokeapiFuerzaBrutaDemo (necesita fallos ~1 min sostenido)";;
  mixto)        echo "│  → mira:    PokeapiTraficoAltoDemo y PokeapiFuerzaBrutaDemo";;
esac
echo "└───────────────────────────────────────────────────────────"

WORK="$(mktemp -d)"
PIDS=()
cleanup_dir() { rm -rf "$WORK" 2>/dev/null || true; }
trap cleanup_dir EXIT

# --- Worker: bucle de curl; cuenta reqs y fallos en su propio archivo ---------
worker() {
  set +e   # subshell propio: curl y aritmética no deben matar el bucle
  local id="$1" f="$WORK/w$1" reqs=0 fails=0
  local names=(pikachu eevee bulbasaur charmander squirtle snorlax gengar mewtwo \
               jigglypuff ditto lucario gyarados dragonite lapras magikarp onix)
  while [[ ! -e "$WORK/stop" ]]; do
    local hacer_login=0
    if [[ "$MODO" == "fuerza-bruta" ]]; then
      hacer_login=1
    elif [[ "$MODO" == "mixto" ]] && (( RANDOM % 4 == 0 )); then
      hacer_login=1
    fi

    if [[ "$hacer_login" == 1 ]]; then
      # usuario EXISTENTE (admin) + password mala => motivo=password_incorrecto
      curl -s -o /dev/null -m "$TIMEOUT" -X POST "$TARGET/api/login" \
        -H 'content-type: application/json' \
        -d "{\"nombre\":\"admin\",\"password\":\"mala-$RANDOM\"}" || fails=$((fails+1))
    else
      local p="${names[RANDOM % ${#names[@]}]}"
      curl -s -o /dev/null -m "$TIMEOUT" "$TARGET/api/pokemon/$p" || fails=$((fails+1))
    fi

    reqs=$((reqs+1))
    printf '%s %s\n' "$reqs" "$fails" > "$f"   # su propio archivo: sin locks
    if [[ "$DELAY" != 0 ]]; then sleep "$DELAY"; fi
  done
  printf '%s %s\n' "$reqs" "$fails" > "$f"
}

for i in $(seq 1 "$WORKERS"); do
  worker "$i" &
  PIDS+=("$!")
done

# --- Bucle principal: mide y muestra req/s en vivo ---------------------------
suma() { cat "$WORK"/w* 2>/dev/null | awk '{r+=$1; f+=$2} END{printf "%d %d", r, f}'; }

PARAR=0
trap 'PARAR=1' INT TERM
inicio=$SECONDS
prev_reqs=0; prev_t=$inicio
while :; do
  sleep 1 || true
  ahora=$SECONDS; transcurrido=$((ahora - inicio))
  read -r total fails <<<"$(suma)"; total=${total:-0}; fails=${fails:-0}
  dt=$((ahora - prev_t)); (( dt < 1 )) && dt=1
  rps_med=$(( (total - prev_reqs) / dt ))
  prev_reqs=$total; prev_t=$ahora
  printf '\r  t=%3ds  reqs=%-8s ~%-5s req/s  fallos=%-5s' "$transcurrido" "$total" "$rps_med" "$fails"
  if [[ "$PARAR" == 1 ]]; then break; fi
  if [[ "$DURACION" != 0 && "$transcurrido" -ge "$DURACION" ]]; then break; fi
done

# --- Parar workers y resumen -------------------------------------------------
touch "$WORK/stop"
kill "${PIDS[@]}" 2>/dev/null || true
wait 2>/dev/null || true
read -r total fails <<<"$(suma)"; total=${total:-0}; fails=${fails:-0}
transcurrido=$((SECONDS - inicio)); (( transcurrido < 1 )) && transcurrido=1
media=$(( total / transcurrido ))
echo
echo "└─ fin: $total peticiones en ${transcurrido}s  (~${media} req/s medio, $fails fallos)"
echo "   Mira las alertas:  http://localhost:9090/alerts   (o la IP pública de prometheus)"
