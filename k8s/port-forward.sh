#!/usr/bin/env bash
# Abre los 3 port-forwards de la demo en segundo plano y deja todo listo en
# localhost (los mismos puertos que usa el guion). Ctrl-C para cerrarlos todos.
set -euo pipefail

NS="${NS:-prometheus-demo}"

echo "Esperando a que los pods estén Ready en el namespace '$NS'..."
kubectl -n "$NS" wait --for=condition=Available deploy/node-exporter deploy/prometheus deploy/alertmanager --timeout=120s

echo "Abriendo port-forwards:"
echo "  - Prometheus    -> http://localhost:9090"
echo "  - node-exporter -> http://localhost:9100/metrics"
echo "  - Alertmanager  -> http://localhost:9093"

pids=()
kubectl -n "$NS" port-forward svc/prometheus    9090:9090 >/tmp/pf-prometheus.log 2>&1 &  pids+=($!)
kubectl -n "$NS" port-forward svc/node-exporter 9100:9100 >/tmp/pf-node.log       2>&1 &  pids+=($!)
kubectl -n "$NS" port-forward svc/alertmanager  9093:9093 >/tmp/pf-alertmanager.log 2>&1 & pids+=($!)

# Opcional: la app pokeapi (solo si está desplegada; el público usa la IP
# pública del svc pokeapi-publico, este forward es para el presentador).
if kubectl -n "$NS" get svc/pokeapi >/dev/null 2>&1; then
  echo "  - pokeapi       -> http://localhost:3000"
  kubectl -n "$NS" port-forward svc/pokeapi 3000:3000 >/tmp/pf-pokeapi.log 2>&1 & pids+=($!)
fi

trap 'echo; echo "Cerrando port-forwards..."; kill "${pids[@]}" 2>/dev/null || true' INT TERM EXIT

echo
echo "Listo. Deja esta terminal abierta durante la charla. Ctrl-C para cerrar."
wait
