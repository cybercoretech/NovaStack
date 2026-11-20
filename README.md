# NovaStack ⚡

[![Crates.io](https://img.shields.io/crates/v/novastack.svg)](https://crates.io/crates/novastack)
[![CI](https://github.com/cybercoretech/NovaStack/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/cybercoretech/NovaStack/actions/workflows/ci.yml)
[![Site](https://img.shields.io/badge/site-novastack--rs.pages.dev-ff9a3c)](https://novastack-rs.pages.dev)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-9b6bff)](#-license)
![Rust 1.88+](https://img.shields.io/badge/rust-1.88%2B-orange)

> **Rust backends, scaffolded.** NovaStack generates Axum backends that compile on the first `cargo run`, then grows them with plugins and a component generator.

Every generated project comes with typed configuration, a central error type that never leaks internals to clients, structured JSON logging, graceful shutdown, a multi-stage Dockerfile, `docker-compose.yml`, a `Justfile` and tests. You can browse the real generated files on the website: **<https://novastack-rs.pages.dev>**.

**🌐 Live demo:** four generated apps, running as `novastack new` creates them (data reset nightly):
[REST + Swagger UI](https://rest-demo.cybercoretech.net/docs/) ·
[HTMX todos](https://htmx-demo.cybercoretech.net/) ·
[WebSocket/SSE](https://live-demo.cybercoretech.net/) ·
gRPC: `grpcurl grpc-demo.cybercoretech.net:443 list`. How it's run: [`demo/`](demo/).

## 🧱 Templates

Pick a starting point with `-t` / `--template` (default `rest`). All templates share the same base, and every plugin works with every template.

| Template | What you get |
|----------|--------------|
| `rest` | JSON API with an OpenAPI spec ([utoipa](https://docs.rs/utoipa)), Swagger UI at `/docs`, and an example `/api/items` CRUD resource stored in the database |
| `htmx` | Server-rendered UI: [Askama](https://docs.rs/askama) templates, HTMX fragment swaps, Tailwind (browser CDN build, for development) |
| `grpc` | [Tonic](https://docs.rs/tonic) gRPC service from a `.proto` file (bundled `protoc`), gRPC health and reflection, on the same port as HTTP |
| `realtime` | WebSocket `/ws`, Server-Sent Events `/events` and `POST /publish`, fed by a broadcast hub in `AppState`, plus a demo page |
| `minimal` | Bare Axum service with a trimmed dependency set |

Databases: `--db postgres` (default), `--db sqlite` or `--db none`. Postgres and SQLite projects get a SQLx pool and embedded migrations.

## 🔌 Plugins

Enable plugins when you create a project (`--plugins auth,otel`) or later (`novastack add redis`). Both paths produce the same code.

| Plugin | Adds |
|--------|------|
| `auth` | User accounts: `/auth/register`, `/auth/login` (JWT) and a protected `/auth/me`, Argon2id password hashes in a `users` table, and an `AuthUser` extractor that guards routes |
| `otel` | Prometheus `/metrics` with per-route request counts and latency |
| `redis` | A `deadpool-redis` pool in `AppState` plus `get` / `set_ex` cache helpers |
| `workers` | A background worker supervisor started at boot, and a `Task` trait for jobs |

Known limits: auth has no rate limiting, email verification or password reset yet, `otel` doesn't export OpenTelemetry traces yet, and plugins guard HTTP routes only (not gRPC or WebSockets). With `--db none`, the example data and accounts live in memory.

## 🛠️ Generate components

`novastack generate <kind> <name>` writes a component and registers it, so the project keeps compiling:

| Kind | Creates | Registers |
|------|---------|-----------|
| `module` | `src/<name>/mod.rs` | `pub mod` in `lib.rs` |
| `handler` | `src/handlers/<name>.rs` with its own router | `handlers/mod.rs`, plus `.nest("/<name>", ...)` in `router.rs` |
| `service` | `src/services/<name>.rs` | `services/mod.rs` (created on first use) |
| `dto` | `src/dtos/<name>.rs` | `dtos/mod.rs` (created on first use) |
| `middleware` | `src/middleware/<name>.rs` (an Axum `from_fn` middleware) | `middleware/mod.rs` |

Names are snake_cased. Rust keywords, built-in module names and existing components are rejected.

## 📦 Installation

Requires **Rust 1.88** or newer.

```bash
cargo install novastack
```

Or the latest unreleased version from the repository:

```bash
cargo install --git https://github.com/cybercoretech/NovaStack
```

## 🚀 Quick start

```bash
# Interactive wizard
novastack new

# Or straight from flags
novastack new my-api --db postgres --plugins auth,otel --no-interactive

cd my-api
cp .env.example .env
docker compose up -d db   # local Postgres
cargo run                 # http://localhost:8080/health
```

More commands:

```bash
novastack add redis                 # add a plugin to an existing project
novastack generate handler orders   # new component, routed at /orders
novastack list                      # templates, plugins, databases (--json for scripts)
novastack doctor                    # check your toolchain (rustc, cargo, docker, sqlx, just)
novastack new my-api --dry-run      # show the files without writing them
```

Generated projects declare `rust-version = "1.88"` and use Cargo's MSRV-aware resolver, so dependency resolution stays buildable on that toolchain.

## ✅ What CI verifies

Every push and pull request runs ([`ci.yml`](.github/workflows/ci.yml)):

- **NovaStack itself:** `rustfmt`, `clippy -D warnings`, tests on stable and on Rust 1.88, and `cargo-deny` (advisories, licenses, sources).
- **Generated projects:** a compile matrix ([`tests/generated_compile.rs`](tests/generated_compile.rs)) that scaffolds every template, database and plugin combination, plus projects grown with `add` and `generate`, then runs `cargo clippy --all-targets -D warnings` and `cargo test` on each one. It runs on stable and on 1.88.
- **macOS and Windows:** NovaStack's tests, plus generated projects (SQLite with every plugin, gRPC with its bundled `protoc`, HTMX) built and tested on both.
- **Docker** (on `main`, weekly and on demand): [`scripts/docker-smoke.sh`](scripts/docker-smoke.sh) builds and runs one project per template and checks it over HTTP:
  - `rest`: Postgres + Redis + all plugins under `docker compose`; CRUD API, Swagger UI, register/login (including 401/409/422 cases) and protected route, metrics, data surviving an app restart
  - `grpc`: a gRPC call over HTTP/2 and HTTP health on the same port, with an SQLite data volume
  - `htmx`: SQLite + auth under Compose; page, add/delete fragments, HTML escaping, todos surviving an app restart
  - `realtime`: Redis + workers under Compose; a message published over HTTP arrives on the SSE stream, WebSocket upgrade
  - `minimal`: health and 404 handling

  Each scenario also checks the container healthcheck and reports the image size.

A weekly run catches new crate releases that would break generated projects, which have no lockfile.

Run the matrix locally (needs network, a few minutes):

```bash
cargo test --test generated_compile -- --ignored
```

## 🗂️ How it works

- `templates_shared/`: the base project, rendered with [minijinja](https://docs.rs/minijinja) for every `new`.
- `templates/<template>/`: the files a template adds. Files ending in `.jinja` are rendered; everything else (Askama HTML, `.proto`, `build.rs`) is copied as is.
- `templates/plugins/`, `templates/generate/`: plugin files and component templates.
- Templates, plugins and generators edit existing files only at **anchor comments** such as `// NOVASTACK:ROUTES` or `# NOVASTACK:DEPENDENCIES`. A missing anchor is reported as an error rather than skipped. Keep the anchors in generated projects if you want to use `add` and `generate` later.
- `novastack.toml` in each generated project records its template, database and plugins.

## 🌐 Website

<https://novastack-rs.pages.dev>, hosted on Cloudflare Pages. It's built from the real binary at deploy time ([`site/tools/build-data.py`](site/tools/build-data.py)), so every file, diff and count on it matches the code. Preview it locally:

```bash
site/tools/build.sh /tmp/novastack-site && python3 -m http.server -d /tmp/novastack-site 8000
```

## 🗺️ Status

NovaStack is early (v0.2.0; see the [changelog](CHANGELOG.md)). Not there yet: OpenTelemetry trace export, gRPC and WebSocket authentication, and a `novastack upgrade` path for existing projects.

## 📜 License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.

---

👻 Code & Design by Darkstardevx 🤖 Generated with [Claude Code](https://claude.com/claude-code)
