#!/usr/bin/env bash
# Installs the NovaStack live demo as rootless Podman Quadlet units for the current user.
#
#   demo/install.sh             install or update, then (re)start everything
#   demo/install.sh --check     only check prerequisites and show what would change
#   demo/install.sh --uninstall stop the demo and remove its unit files (keeps data + secrets)
#
# Nothing here needs sudo. Units start at boot through user lingering
# (`loginctl show-user "$USER" -p Linger` must say yes).
#
# Secrets live in ~/.config/novastack-demo/ (mode 600) and are never printed:
#   tunnel.env    TUNNEL_TOKEN=...  (you create it from the Cloudflare dashboard)
#   postgres.env, rest.env, htmx.env, realtime.env, grpc.env  (generated here once)
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
config="$HOME/.config/novastack-demo"
quadlet_dir="$HOME/.config/containers/systemd/novastack-demo"
user_units="$HOME/.config/systemd/user"
apps=(rest htmx realtime grpc)
mode=${1:-install}

say() { printf '%s\n' "$*"; }
fail() { printf 'error: %s\n' "$*" >&2; exit 1; }

check_prereqs() {
  command -v podman >/dev/null || fail "podman is not installed"
  [ -x /usr/lib/podman/quadlet ] || [ -x /usr/libexec/podman/quadlet ] || fail "podman quadlet generator not found"
  [ "$(loginctl show-user "$USER" -p Linger --value 2>/dev/null)" = yes ] \
    || say "warning: lingering is off, so the demo only runs while you are logged in (sudo loginctl enable-linger $USER)"
  local token="$config/tunnel.env"
  [ -s "$token" ] || fail "$token is missing: create it with one line, TUNNEL_TOKEN=<token from the Cloudflare dashboard>"
  # Shape check only; the value is never printed.
  awk -F= 'NR==1 && $1=="TUNNEL_TOKEN" && substr($0,14,3)=="eyJ" && length($0) > 100 {ok=1} END {exit !ok}' "$token" \
    || fail "$token doesn't look like TUNNEL_TOKEN=eyJ... (the long token, not the tunnel ID)"
  [ "$(stat -c %a "$token")" = 600 ] || say "warning: chmod 600 $token"
  say "prerequisites ok"
}

# Writes a secrets file once; later runs keep it, so passwords stay stable.
write_secret_file() { # name, content
  local file="$config/$1"
  if [ -e "$file" ]; then say "  keep    $file"; return; fi
  if [ "$mode" = --check ]; then say "  create  $file"; return; fi
  (umask 077; printf '%s\n' "$2" > "$file")
  say "  create  $file"
}

secrets() {
  mkdir -p "$config" && chmod 700 "$config"
  local pg_password jwt
  if [ -e "$config/postgres.env" ]; then
    pg_password=$(sed -n 's/^POSTGRES_PASSWORD=//p' "$config/postgres.env")
  else
    pg_password=$(openssl rand -hex 24)
  fi
  jwt=$(openssl rand -hex 32)
  local common="HOST=0.0.0.0
PORT=8080
RUST_LOG=info"
  write_secret_file postgres.env "POSTGRES_USER=novastack
POSTGRES_PASSWORD=$pg_password
POSTGRES_DB=rest"
  write_secret_file rest.env "$common
DATABASE_URL=postgres://novastack:$pg_password@demo-postgres:5432/rest
DATABASE_MAX_CONNECTIONS=5
JWT_SECRET=$jwt"
  write_secret_file htmx.env "$common
DATABASE_URL=postgres://novastack:$pg_password@demo-postgres:5432/htmx
DATABASE_MAX_CONNECTIONS=5
JWT_SECRET=$(openssl rand -hex 32)"
  write_secret_file realtime.env "$common"
  write_secret_file grpc.env "$common"
}

copy() { # source, destination
  if [ -e "$2" ] && cmp -s "$1" "$2"; then return; fi
  say "  install $2"
  [ "$mode" = --check ] || install -m 644 -D "$1" "$2"
}

install_units() {
  copy "$here/postgres/init.sql" "$config/init.sql"
  for unit in "$here"/quadlet/*; do copy "$unit" "$quadlet_dir/$(basename "$unit")"; done
  for unit in "$here"/systemd/*; do copy "$unit" "$user_units/$(basename "$unit")"; done
}

start() {
  systemctl --user daemon-reload
  # Quadlet units start at boot through their [Install] section; start them now too.
  systemctl --user restart demo-postgres.service
  for app in "${apps[@]}"; do systemctl --user restart "demo-$app.service"; done
  systemctl --user restart demo-cloudflared.service
  systemctl --user enable --now novastack-demo-reset.timer podman-auto-update.timer
  say "running:"
  systemctl --user --no-pager --plain list-units 'demo-*' 'novastack-demo-*' | sed 's/^/  /'
}

uninstall() {
  systemctl --user disable --now novastack-demo-reset.timer 2>/dev/null || true
  systemctl --user stop demo-cloudflared.service "${apps[@]/#/demo-}" demo-postgres.service 2>/dev/null || true
  for unit in "$here"/quadlet/*; do rm -f "$quadlet_dir/$(basename "$unit")"; done
  rmdir "$quadlet_dir" 2>/dev/null || true
  for unit in "$here"/systemd/*; do rm -f "$user_units/$(basename "$unit")"; done
  systemctl --user daemon-reload
  say "demo stopped and unit files removed. Kept: $config (secrets) and the novastack-demo-pg volume (data)."
}

case "$mode" in
  install) check_prereqs; secrets; install_units; start ;;
  --check) check_prereqs; secrets; install_units; say "check only: nothing was changed" ;;
  --uninstall) uninstall ;;
  *) fail "usage: demo/install.sh [--check|--uninstall]" ;;
esac
