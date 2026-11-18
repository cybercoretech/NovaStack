#!/usr/bin/env bash
# Builds and runs NovaStack-generated projects in Docker and checks them over HTTP.
#
#   scripts/docker-smoke.sh <path-to-novastack-binary> <work-dir>
#
# 1. rest + postgres + all plugins via `docker compose up` (app, postgres, redis)
# 2. grpc + sqlite as a single container (bundled protoc, sqlite data volume)
# SC2329: cleanup functions run via `trap`. SC2016: `sh -c` strings expand in that shell.
# shellcheck disable=SC2329,SC2016
set -euo pipefail

novastack=${1:?usage: docker-smoke.sh <novastack-binary> <work-dir>}
work=${2:?usage: docker-smoke.sh <novastack-binary> <work-dir>}
mkdir -p "$work"
cd "$work"

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

echo "== rest + postgres + all plugins (docker compose)"
"$novastack" new smoke-rest -t rest --db postgres --plugins auth,otel,redis,workers \
  --no-interactive --skip-hooks >/dev/null
(
  cd smoke-rest
  cp .env.example .env
  sed -i "s/^JWT_SECRET=.*/JWT_SECRET=$(openssl rand -hex 32)/" .env
  cleanup() { docker compose logs app | tail -n 50 || true; docker compose down -v --remove-orphans; }
  trap cleanup EXIT
  docker compose up -d --build
  wait_for http://localhost:8080/health 90

  expect "health" sh -c 'curl -fsS localhost:8080/health | grep -q "\"ok\""'
  expect "swagger ui" curl -fsS -o /dev/null localhost:8080/docs/
  expect "create item" sh -c "curl -fsS -H 'content-type: application/json' -d '{\"name\":\"Widget\"}' localhost:8080/api/items | grep -q Widget"
  token=$(curl -fsS -H 'content-type: application/json' -d '{"username":"smoke","password":"pw"}' localhost:8080/auth/login | sed -E 's/.*"token":"([^"]+)".*/\1/')
  expect "auth/me with token" sh -c "curl -fsS -H 'Authorization: Bearer $token' localhost:8080/auth/me | grep -q smoke"
  expect "auth/me without token is 401" sh -c '[ "$(curl -s -o /dev/null -w %{http_code} localhost:8080/auth/me)" = 401 ]'
  expect "metrics" sh -c 'curl -fsS localhost:8080/metrics | grep -q http_requests_total'
  expect "container healthcheck" sh -c '[ "$(docker inspect -f {{.State.Health.Status}} $(docker compose ps -q app))" = healthy ] || { sleep 20; [ "$(docker inspect -f {{.State.Health.Status}} $(docker compose ps -q app))" = healthy ]; }'
)

echo "== grpc + sqlite (single container)"
"$novastack" new smoke-grpc -t grpc --db sqlite --no-interactive --skip-hooks >/dev/null
(
  cd smoke-grpc
  docker build -t novastack-smoke-grpc .
  cleanup() { docker logs smoke-grpc 2>&1 | tail -n 50 || true; docker rm -f smoke-grpc >/dev/null 2>&1 || true; docker volume rm smoke-grpc-data >/dev/null 2>&1 || true; }
  trap cleanup EXIT
  docker run -d --name smoke-grpc -p 8081:8080 -v smoke-grpc-data:/app/data \
    -e DATABASE_URL='sqlite:///app/data/smoke.db?mode=rwc' novastack-smoke-grpc
  wait_for http://localhost:8081/health 60

  printf '\x00\x00\x00\x00\x0b\x0a\x09NovaStack' > hello.bin
  expect "grpc SayHello" sh -c "curl -fsS --http2-prior-knowledge -H 'content-type: application/grpc' -H 'te: trailers' --data-binary @hello.bin localhost:8081/greeter.v1.Greeter/SayHello | grep -aq 'Hello, NovaStack!'"
)

echo "docker smoke: all checks passed"
