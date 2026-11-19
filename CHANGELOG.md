# Changelog

All notable changes to NovaStack are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses
[Semantic Versioning](https://semver.org/).

## [0.2.0] - 2026-11-19

Generated example code now stores data and checks passwords for real.

### Added

- `auth` plugin: user accounts. `POST /auth/register` creates a user (username 3–64
  characters without spaces, password 8–128 characters; 422 otherwise, 409 when the
  username is taken) and stores an Argon2id hash. `POST /auth/login` checks the password
  and issues a JWT whose subject is the user id. `GET /auth/me` returns the account's id
  and username. A `UserStore` in `AppState` is backed by a new `users` table and migration
  (Postgres or SQLite), or by memory with `--db none`.
- `AppError::Conflict` (409) in generated projects.
- Generated tests run against an in-memory SQLite database for SQLite projects and against
  `DATABASE_URL` for Postgres projects (skipped when it isn't set). The auth plugin ships
  tests for registration, login, wrong passwords, unknown users and duplicates.

### Changed

- The `rest` example resource (`/api/items`) and the `htmx` todo list are stored in
  database tables (`items`, `todos`) with SQLx when the project has a database. Projects
  with `--db none` keep the in-memory stores. Item and todo ids are now `i64`.
- `api::routes()` and `web::routes()` take their store (`ItemStore` / `TodoStore`), built
  from `AppState` in `router.rs`.
- Login checks are uniform: unknown usernames and wrong passwords both return the same
  401 and both run one Argon2 verification. Hashing runs on the blocking thread pool.
- Archetype and plugin files under `src/db/` are skipped for projects without a database,
  like the shared ones.

### Removed

- The `/auth/login` stub that issued a token for any non-empty credentials.

### Verified

- CI also builds and tests an `htmx` project on Postgres. The Docker smoke tests register
  and log in (including 401, 409 and 422 cases) and restart the app to check that items,
  accounts and todos persist.

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

[0.2.0]: https://github.com/cybercoretech/NovaStack/releases/tag/v0.2.0
[0.1.0]: https://github.com/cybercoretech/NovaStack/releases/tag/v0.1.0
