# Changelog

All notable changes to NovaStack are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses
[Semantic Versioning](https://semver.org/).

## [0.1.0] - 2026-11-19

First release on crates.io.

### Added

- `novastack new`: scaffolds an Axum 0.8 backend with typed configuration, a central
  `AppError`, structured JSON logging, graceful shutdown, a multi-stage Dockerfile,
  `docker-compose.yml`, a `Justfile`, `.env.example` and tests. Interactive wizard, or
  flags with `--no-interactive`; `--dry-run` lists files without writing them.
- Five templates (`-t`):
  - `rest`: OpenAPI spec (utoipa) and Swagger UI at `/docs`, example `/api/items` CRUD.
  - `htmx`: Askama templates, HTMX fragment swaps, Tailwind (browser CDN, development).
  - `grpc`: Tonic service from a `.proto` file with a bundled `protoc`, gRPC health and
    reflection, served on the same port as HTTP.
  - `realtime`: WebSocket `/ws`, Server-Sent Events `/events`, `POST /publish`, broadcast
    hub in `AppState`.
  - `minimal`: bare Axum service with a trimmed dependency set.
- Databases: `--db postgres` (default), `sqlite` or `none`, with SQLx pools and embedded
  migrations.
- Plugins, at creation (`--plugins`) or later (`novastack add`), producing the same code:
  - `auth`: JWT issuing and verification, Argon2 password hashing, an `AuthUser` extractor,
    `/auth/login` and a protected `/auth/me`.
  - `otel`: Prometheus `/metrics` with per-route request counts and latency.
  - `redis`: `deadpool-redis` pool in `AppState` with cache helpers.
  - `workers`: background worker supervisor and a `Task` trait.
- `novastack generate {module|handler|service|dto|middleware} <name>`: writes a component
  and registers it (module declarations; handlers are also routed).
- `novastack list` (and `--json`), `novastack doctor`.
- Generated projects pin `rust-version = "1.88"` and use the MSRV-aware resolver.

### Verified

- CI builds every template, database and plugin combination (plus projects grown with
  `add` and `generate`) and runs `cargo clippy -D warnings` and `cargo test` on each, on
  stable and on Rust 1.88. Docker images are built and smoke-tested (`docker compose` with
  Postgres, Redis and all plugins; gRPC image) on `main` and weekly.

### Known limitations

- The generated `/auth/login` is a stub that accepts any non-empty credentials; wire it to
  your user store before shipping.
- The `otel` plugin serves Prometheus metrics; OpenTelemetry trace export is not
  implemented yet.
- Plugins guard HTTP routes only (no gRPC interceptor or WebSocket authentication).
- The REST and HTMX example resources keep data in memory.

[0.1.0]: https://github.com/cybercoretech/NovaStack/releases/tag/v0.1.0
