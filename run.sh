#!/bin/sh
set -e

# ---- shutdown helper --------------------------------------
stop() {
  echo "Stopping backend…"
  kill -TERM "$BACKEND_PID" 2>/dev/null || true
  echo "Stopping Nginx…"
  kill -TERM "$NGINX_PID" 2>/dev/null || true
  wait "$BACKEND_PID" "$NGINX_PID" 2>/dev/null || true
  exit 0
}
trap stop INT TERM

# ---- start backend --------------------------------------------------
echo "Starting backend…"
/usr/local/bin/backend_simple_web &
BACKEND_PID=$!

# ---- start nginx ----------------------------------------------------
echo "Starting Nginx…"
nginx -g 'daemon off;' &
NGINX_PID=$!

# ---- wait until either process dies --------------------------------
while kill -0 "$BACKEND_PID" 2>/dev/null && kill -0 "$NGINX_PID" 2>/dev/null
do
  sleep 1
done
stop
