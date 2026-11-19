#!/usr/bin/env bash
# Builds and runs NovaStack-generated projects in Docker and checks them over HTTP.
#
#   scripts/docker-smoke.sh <novastack-binary> <work-dir> <scenario>
#
# Scenarios (one per template):
#   rest      rest + postgres + all plugins, docker compose (app, postgres, redis)
#   grpc      grpc + sqlite, single container (bundled protoc, sqlite data volume)
#   htmx      htmx + sqlite + auth, docker compose (sqlite volume, .env secrets)
#   realtime  realtime + redis + workers, docker compose (app, redis)
#   minimal   minimal, single container
#
# SMOKE_URL=<base-url> runs only a scenario's HTTP checks against an already running
# server (no Docker), which is how the checks are validated without a Docker daemon.
# SC2329: cleanup functions run via `trap`. SC2016: `sh -c` strings expand in that shell.
# shellcheck disable=SC2329,SC2016
set -euo pipefail

novastack=${1:?usage: docker-smoke.sh <novastack-binary> <work-dir> <scenario>}
work=${2:?usage: docker-smoke.sh <novastack-binary> <work-dir> <scenario>}
scenario=${3:?usage: docker-smoke.sh <novastack-binary> <work-dir> <scenario>}

wait_for() { # url, attempts
  for _ in $(seq 1 "${2:-60}"); do
    if curl -fsS "$1" >/dev/null 2>&1; then return 0; fi
    sleep 2
  done
  echo "timed out waiting for $1" >&2
  return 1
}

expect() { # description, command...
  local what=$1; shift
  if "$@"; then echo "  ok   $what"; else echo "  FAIL $what" >&2; exit 1; fi
}

healthy() { # container id
  for _ in $(seq 1 15); do
    [ "$(docker inspect -f '{{.State.Health.Status}}' "$1")" = healthy ] && return 0
    sleep 3
  done
  return 1
}

image_size() { # container id
  local image
  image=$(docker inspect -f '{{.Image}}' "$1")
  echo "  size $(docker image inspect -f '{{.Size}}' "$image" | awk '{printf "%.1f MB", $1 / 1000000}') image"
}

# ---------- HTTP checks per scenario (base URL as $1) ----------

checks_rest() {
  local b=$1 token
  expect "health" sh -c "curl -fsS $b/health | grep -q '\"ok\"'"
  expect "swagger ui" curl -fsS -o /dev/null "$b/docs/"
  expect "create item" sh -c "curl -fsS -H 'content-type: application/json' -d '{\"name\":\"Widget\"}' $b/api/items | grep -q Widget"
  token=$(curl -fsS -H 'content-type: application/json' -d '{"username":"smoke","password":"pw"}' "$b/auth/login" | sed -E 's/.*"token":"([^"]+)".*/\1/')
  expect "auth/me with token" sh -c "curl -fsS -H 'Authorization: Bearer $token' $b/auth/me | grep -q smoke"
  expect "auth/me without token is 401" sh -c "[ \"\$(curl -s -o /dev/null -w %{http_code} $b/auth/me)\" = 401 ]"
  expect "metrics" sh -c "curl -fsS $b/metrics | grep -q http_requests_total"
}

checks_grpc() {
  local b=$1
  printf '\x00\x00\x00\x00\x0b\x0a\x09NovaStack' > "$work/hello.bin"
  expect "grpc SayHello" sh -c "curl -fsS --http2-prior-knowledge -H 'content-type: application/grpc' -H 'te: trailers' --data-binary @$work/hello.bin $b/greeter.v1.Greeter/SayHello | grep -aq 'Hello, NovaStack!'"
  expect "http health on the same port" sh -c "curl -fsS $b/health | grep -q '\"ok\"'"
}

checks_htmx() {
  local b=$1 fragment="$work/fragment.html" id
  expect "page renders with htmx" sh -c "curl -fsS $b/ | grep -q 'hx-post=\"/todos\"'"
  curl -fsS -d 'title=<b>Milk</b>' "$b/todos" > "$fragment"
  expect "add returns an <li> fragment" grep -q '<li' "$fragment"
  expect "fragment escapes html" sh -c "grep -q Milk '$fragment' && ! grep -q '<b>' '$fragment'"
  id=$(sed -nE 's|.*hx-delete="/todos/([0-9]+)".*|\1|p' "$fragment" | head -1)
  expect "page lists the new todo" sh -c "curl -fsS $b/ | grep -q Milk"
  expect "delete removes it" sh -c "[ \"\$(curl -s -o /dev/null -w %{http_code} -X DELETE $b/todos/$id)\" = 200 ]"
  expect "blank title is rejected (422)" sh -c "[ \"\$(curl -s -o /dev/null -w %{http_code} -d 'title=+' $b/todos)\" = 422 ]"
}

checks_realtime() {
  local b=$1 events="$work/events.txt"
  expect "demo page" sh -c "curl -fsS $b/ | grep -q 'Realtime demo'"
  curl -sN --max-time 6 "$b/events" > "$events" &
  local sse=$!
  sleep 1
  expect "publish accepted (202)" sh -c "[ \"\$(curl -s -o /dev/null -w %{http_code} -H 'content-type: application/json' -d '{\"message\":\"smoke-hello\"}' $b/publish)\" = 202 ]"
  wait "$sse" || true
  expect "SSE stream delivered the message" grep -q 'data: smoke-hello' "$events"
  expect "websocket upgrade (101)" sh -c "[ \"\$(curl -s -o /dev/null -w %{http_code} --max-time 3 -H 'Connection: Upgrade' -H 'Upgrade: websocket' -H 'Sec-WebSocket-Version: 13' -H 'Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==' $b/ws)\" = 101 ]"
}

checks_minimal() {
  local b=$1
  expect "health" sh -c "curl -fsS $b/health | grep -q '\"ok\"'"
  expect "unknown route is 404" sh -c "[ \"\$(curl -s -o /dev/null -w %{http_code} $b/nope)\" = 404 ]"
}

# ---------- Checks only (no Docker) ----------
if [ -n "${SMOKE_URL:-}" ]; then
  mkdir -p "$work"
  echo "== $scenario checks against $SMOKE_URL"
  "checks_$scenario" "$SMOKE_URL"
  echo "checks passed"
  exit 0
fi

# ---------- Docker runs ----------
mkdir -p "$work"
cd "$work"

compose_scenario() { # name, scenario, novastack args...
  local name=$1 kind=$2; shift 2
  "$novastack" new "$name" "$@" --no-interactive --skip-hooks >/dev/null
  cd "$name"
  cp .env.example .env
  if grep -q '^JWT_SECRET=' .env; then
    sed -i "s/^JWT_SECRET=.*/JWT_SECRET=$(openssl rand -hex 32)/" .env
  fi
  cleanup() { docker compose logs app | tail -n 50 || true; docker compose down -v --remove-orphans; }
  trap cleanup EXIT
  docker compose up -d --build
  wait_for http://localhost:8080/health 90
  "checks_$kind" http://localhost:8080
  expect "container healthcheck" healthy "$(docker compose ps -q app)"
  image_size "$(docker compose ps -q app)"
}

# Script-level, not local: the EXIT trap runs after container_scenario has returned.
container=""

container_scenario() { # name, scenario, extra docker args (one string), novastack args...
  local name=$1 kind=$2 extra=$3; shift 3
  container=$name
  "$novastack" new "$name" "$@" --no-interactive --skip-hooks >/dev/null
  cd "$name"
  docker build -t "novastack-smoke-$name" .
  cleanup() { docker logs "$container" 2>&1 | tail -n 50 || true; docker rm -f "$container" >/dev/null 2>&1 || true; docker volume rm "$container-data" >/dev/null 2>&1 || true; }
  trap cleanup EXIT
  # shellcheck disable=SC2086
  docker run -d --name "$name" -p 8081:8080 $extra "novastack-smoke-$name"
  wait_for http://localhost:8081/health 60
  "checks_$kind" http://localhost:8081
  expect "container healthcheck" healthy "$name"
  image_size "$name"
}

echo "== $scenario"
case "$scenario" in
  rest)     compose_scenario smoke-rest rest -t rest --db postgres --plugins auth,otel,redis,workers ;;
  htmx)     compose_scenario smoke-htmx htmx -t htmx --db sqlite --plugins auth ;;
  realtime) compose_scenario smoke-realtime realtime -t realtime --db none --plugins redis,workers ;;
  grpc)     container_scenario smoke-grpc grpc "-v smoke-grpc-data:/app/data -e DATABASE_URL=sqlite:///app/data/smoke.db?mode=rwc" -t grpc --db sqlite ;;
  minimal)  container_scenario smoke-minimal minimal "" -t minimal --db none ;;
  *) echo "unknown scenario: $scenario" >&2; exit 2 ;;
esac
echo "docker smoke ($scenario): all checks passed"
