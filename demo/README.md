# 🌐 NovaStack live demo

Four NovaStack-generated apps, running exactly as `novastack new` creates them:

| URL | Template | What to try |
|---|---|---|
| https://rest-demo.cybercoretech.net/docs | `rest` + Postgres + `auth`, `otel` | Swagger UI: create items, register, log in, call `/auth/me` |
| https://htmx-demo.cybercoretech.net | `htmx` + Postgres + `auth` | Add and delete todos (HTMX fragment swaps) |
| https://live-demo.cybercoretech.net | `realtime` | WebSocket + Server-Sent Events demo page |
| `grpc-demo.cybercoretech.net:443` | `grpc` | `grpcurl grpc-demo.cybercoretech.net:443 list` (reflection is on) |

Data is wiped every night at 04:00 (host time). Don't put anything real in it.

## How it runs

```
visitor ─HTTPS─▶ Cloudflare ◀══ outbound tunnel ══ demo-cloudflared ─┐  network novastack-demo (internal)
                                                                       ├─▶ demo-rest ─┐
                                                                       ├─▶ demo-htmx ─┼─ network novastack-demo-db (internal) ─▶ demo-postgres
                                                                       ├─▶ demo-realtime
                                                                       └─▶ demo-grpc
```

- **Rootless Podman, Quadlet units** (`quadlet/`): systemd user services owned by an ordinary user. No sudo, no root daemon, no changes under `/etc`. They start at boot through user lingering.
- **Nothing listens on the host.** No unit publishes a port. The only way in is the Cloudflare Tunnel, which `cloudflared` opens outbound. Postgres sits on a separate internal network that the tunnel container isn't on.
- **Hardened containers:** read-only root filesystems, all capabilities dropped (Postgres keeps the five its entrypoint needs), `no-new-privileges`, and memory, CPU and process limits.
- **Images** come from GHCR (`ghcr.io/cybercoretech/novastack-demo-*`). [`demo-images.yml`](../.github/workflows/demo-images.yml) builds them with the released NovaStack, runs `scripts/docker-smoke.sh` against each one under the same hardening, and publishes `:latest` plus the version. `podman-auto-update.timer` on the host pulls a new `:latest` daily and restarts what changed.
- **Nightly reset:** `novastack-demo-reset.timer` truncates the `items`, `todos` and `users` tables.

## One-time setup

### Cloudflare (dashboard)

1. **Tunnel:** Zero Trust → Networks → Tunnels → create `novastack-demo` (type Cloudflared). Copy only the token (the `eyJ…` string from the install command) into `~/.config/novastack-demo/tunnel.env` as `TUNNEL_TOKEN=eyJ…`, mode 600. Don't run Cloudflare's install command: `cloudflared` runs as a container here.
2. **Routes** (tunnel → Routes → Published application), service type HTTP. They match top to bottom:

   | # | Hostname | Path | Service |
   |---|---|---|---|
   | 1 | `rest-demo.cybercoretech.net` | `^/metrics` | `http_status:404` |
   | 2 | `rest-demo.cybercoretech.net` | | `http://demo-rest:8080` |
   | 3 | `htmx-demo.cybercoretech.net` | | `http://demo-htmx:8080` |
   | 4 | `live-demo.cybercoretech.net` | | `http://demo-realtime:8080` |
   | 5 | `grpc-demo.cybercoretech.net` | | `http://demo-grpc:8080`, with **TLS → HTTP2 connection** on |

   Use one-level names (`rest-demo`, not `rest.demo`). The free Universal SSL certificate only covers `*.cybercoretech.net`.
3. **gRPC:** zone `cybercoretech.net` → Network → gRPC **On**.
4. **Rate limit** (Security → WAF → Rate limiting rules, one rule on the free plan):
   - Expression: `(http.host in {"rest-demo.cybercoretech.net" "htmx-demo.cybercoretech.net"} and (starts_with(http.request.uri.path, "/auth/") or http.request.method eq "POST"))`
   - Characteristics: IP. 20 requests per 10 seconds. Action: Block for 10 seconds.

### GitHub

After the first publish, set each `novastack-demo-*` package to **public** (org → Packages → package → Package settings → Change visibility). The demo host then pulls without a login.

### The host

```bash
demo/install.sh --check   # prerequisites, and which files it would create
demo/install.sh           # install or update the units, generate secrets once, start everything
```

`install.sh` writes the database password and JWT secrets to `~/.config/novastack-demo/*.env` (mode 600) the first time and keeps them afterwards. It never prints them.

## Operating it

| Task | Command |
|---|---|
| Status | `systemctl --user list-units 'demo-*' 'novastack-demo-*'` |
| Logs | `journalctl --user -u demo-rest -f` (or `podman logs demo-rest`) |
| Tunnel connected? | `journalctl --user -u demo-cloudflared \| grep -i registered` |
| Pull new images now | `systemctl --user start podman-auto-update.service` |
| Reset data now | `systemctl --user start novastack-demo-reset.service` |
| Next reset | `systemctl --user list-timers novastack-demo-reset.timer` |
| Update units after a `git pull` | `demo/install.sh` |
| Stop and remove | `demo/install.sh --uninstall` (keeps the secrets and the `novastack-demo-pg` volume) |

## Troubleshooting

- **Tunnel won't connect, `cloudflared` logs `dial udp 10.89.x.1:53: i/o timeout`, often after the host changed networks.** Rootless Podman routes every bridge-networked container through one shared network namespace, which copies the host's routes when it's created. After a network change it can be left without a default route. Check with `podman unshare --rootless-netns ip route`: there must be a `default via …` line. To fix it, stop every rootless container that uses a bridge network (the demo units plus any other stacks, e.g. `systemctl --user stop 'demo-*'`), confirm `podman ps` is empty, then start them again. The namespace is rebuilt with the current routes.
- **`/metrics` is public.** In the tunnel's Routes, the `^/metrics` → `http_status:404` route must be above the plain `rest-demo` route (routes match top to bottom). `cloudflared`'s log line `Updated to new configuration` lists the order it uses.
- **An app keeps restarting:** `journalctl --user -u demo-<app>` and `podman logs demo-<app>`. Its settings are in `~/.config/novastack-demo/<app>.env`.

## Checks

- **CI, every PR** (`demo units` job): Podman 5.8's Quadlet generator accepts every unit, all six services are generated, no unit publishes a port, `install.sh` passes shellcheck, the reset units pass `systemd-analyze verify`, and the reset unit's SQL (after systemd's `$$` unescaping) runs against Postgres, both before and after the tables exist.
- **Each image build:** the image runs read-only with no capabilities (and Postgres where needed) and passes the same HTTP checks as the Docker smoke tests.
- **After deploying:** `curl -4 https://rest-demo.cybercoretech.net/health` and the other hostnames, plus `grpcurl grpc-demo.cybercoretech.net:443 list`.
