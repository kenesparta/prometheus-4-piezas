#!/usr/bin/env bash
# =============================================================================
# Regenera los 3 códigos QR de la charla (app, Prometheus, Grafana) y sincroniza
# las IPs impresas en el deck. Úsalo tras recrear el cluster (montar-charla.sh),
# cuando el LoadBalancer reparte IPs públicas nuevas.
#
# Qué toca:
#   - slides-typst/assets/qr-{app,prometheus,grafana}.svg   (los QR embebidos)
#   - slides-typst/slides/02-qr-app.typ / 03-qr-prometheus.typ / 04-qr-grafana.typ
#   - slides-typst/theme.typ            (#let url-app)
#   - slides.md                         (slide "Ahora, ustedes")
#   - recompila slides-typst/slides.pdf
#
# IPs: por defecto se leen del cluster (Services LoadBalancer del namespace
# prometheus-demo, igual que montar-charla.sh). Puedes forzarlas:
#   ./gen-qr.sh                                   # auto desde kubectl
#   ./gen-qr.sh --app 1.2.3.4 --prometheus 5.6.7.8 --grafana 9.10.11.12
#   APP_IP=1.2.3.4 PROM_IP=5.6.7.8 GRAF_IP=9.10.11.12 ./gen-qr.sh
#
# Nota: el QR del slide proyectable en claude.ai es aparte; este script no lo
# republica (regenéralo pidiéndoselo a Claude si lo necesitas).
#
# Requisitos: python3 (crea un venv local .qrvenv con segno la 1.ª vez),
# typst en el PATH. Para el modo auto, kubectl apuntando al cluster.
# =============================================================================
set -euo pipefail

NS="prometheus-demo"
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"   # slides-typst/
REPO="$(cd "$DIR/.." && pwd)"                          # raíz del repo
ASSETS="$DIR/assets"
SLIDES="$DIR/slides"

# --- IPs: flags > env > kubectl ----------------------------------------------
APP_IP="${APP_IP:-}"; PROM_IP="${PROM_IP:-}"; GRAF_IP="${GRAF_IP:-}"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --app)        APP_IP="$2";  shift 2;;
    --prometheus) PROM_IP="$2"; shift 2;;
    --grafana)    GRAF_IP="$2"; shift 2;;
    -h|--help)    sed -n '2,30p' "$0" | sed 's/^# \{0,1\}//'; exit 0;;
    *) echo "Opción desconocida: $1  (usa -h para la ayuda)"; exit 1;;
  esac
done

svc_ip() {  # svc_ip NOMBRE-SERVICIO
  kubectl -n "$NS" get svc "$1" \
    -o jsonpath='{.status.loadBalancer.ingress[0].ip}' 2>/dev/null || true
}
[[ -z "$APP_IP"  ]] && APP_IP="$(svc_ip pokeapi-publico)"
[[ -z "$PROM_IP" ]] && PROM_IP="$(svc_ip prometheus-publico)"
[[ -z "$GRAF_IP" ]] && GRAF_IP="$(svc_ip grafana)"

falta=0
for pair in "app:$APP_IP" "Prometheus:$PROM_IP" "Grafana:$GRAF_IP"; do
  if [[ -z "${pair#*:}" ]]; then
    echo "!! Sin IP para ${pair%%:*}. Pásala con --... o revisa: kubectl -n $NS get svc"
    falta=1
  fi
done
[[ "$falta" -eq 1 ]] && exit 1

echo "==> IPs:"
echo "    app        http://$APP_IP"
echo "    prometheus http://$PROM_IP"
echo "    grafana    http://$GRAF_IP"

# --- Entorno de QR: venv local con segno (una sola vez) ----------------------
VENV="$DIR/.qrvenv"
PY="$VENV/bin/python"
if [[ ! -x "$PY" ]]; then
  echo "==> Preparando entorno de QR (venv + segno, solo la 1.ª vez)"
  python3 -m venv "$VENV"
  "$VENV/bin/pip" install --quiet --disable-pip-version-check segno
fi

# --- 1) Regenerar los 3 SVG --------------------------------------------------
echo "==> 1/3  Generando SVG en assets/"
mkdir -p "$ASSETS"
"$PY" - "$ASSETS" "$APP_IP" "$PROM_IP" "$GRAF_IP" <<'PY'
import sys, segno
assets, app, prom, graf = sys.argv[1:5]
for name, ip in (("app", app), ("prometheus", prom), ("grafana", graf)):
    url = f"http://{ip}"
    segno.make(url, error='m').save(
        f"{assets}/qr-{name}.svg", scale=16, border=4, dark="#111111", light="#ffffff")
    print(f"    qr-{name}.svg -> {url}")
PY

# --- 2) Sincronizar las IPs impresas en el deck ------------------------------
echo "==> 2/3  Actualizando IPs en los .typ y slides.md"
# En cada slide QR: reemplaza el [http://...] mono por la IP nueva.
patch_slide() {  # patch_slide ARCHIVO IP
  perl -0pi -e "s{\\[http://[^\\]]+\\]}{[http://$2]}g" "$1"
}
patch_slide "$SLIDES/02-qr-app.typ"        "$APP_IP"
patch_slide "$SLIDES/03-qr-prometheus.typ" "$PROM_IP"
patch_slide "$SLIDES/04-qr-grafana.typ"    "$GRAF_IP"
# theme.typ: #let url-app = "http://<lo que sea>"
perl -0pi -e "s{(#let url-app = \")[^\"]*(\")}{\${1}http://$APP_IP\${2}}" "$DIR/theme.typ"
# slides.md: la slide "Ahora, ustedes" -> ## `http://<lo que sea>`
perl -0pi -e "s{^(## \`)http://[^\`]*(\`)}{\${1}http://$APP_IP\${2}}m" "$REPO/slides.md"
echo "    theme.typ (url-app) y slides.md sincronizados"

# --- 3) Recompilar el deck ---------------------------------------------------
echo "==> 3/3  Recompilando slides.pdf"
if command -v typst >/dev/null 2>&1; then
  ( cd "$DIR" && typst compile --font-path fonts slides.typ slides.pdf )
  echo "    slides.pdf listo"
else
  echo "!!  typst no está en el PATH; recompila a mano: make -C slides-typst"
fi

echo "✅ Hecho. Revisa slides.pdf (páginas 2-4) antes de proyectar."
