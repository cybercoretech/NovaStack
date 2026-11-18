# AGENT_HANDOFF.md

## Project
- **Name:** NovaStack (`novastack`), crate lib name `novastack_core`
- **Purpose:** CLI that scaffolds Rust web backends (Axum 0.8, optional SQLx Postgres/SQLite) with optional plugins (auth, otel, redis, workers) and generates components inside existing projects.
- **Repository:** https://github.com/cybercoretech/NovaStack (local: `~/Devspace/Cybercore/tooling/novastack`)
- **Workflow:** always branch + PR; never stack PRs; check CI before merging.

## Current state (2026-11-18)
Steps 1–2 merged in PR #1. Step 3 (real archetypes) on branch `feat/archetypes`:
- All five `-t` archetypes are real (see table below); each ships tests in the generated project.
- Verified: compile matrix 12/12 (every archetype alone and with sqlite + all plugins; minimal grown via `add` + `generate`), every archetype + all plugins on Rust 1.88, and smoke runs of each server (REST CRUD + Swagger UI, HTMX page + fragments, gRPC SayHello/health over h2c, SSE + WebSocket upgrade).

Earlier (PR #1, steps 1–2):
- Every generated project compiles clean under `cargo clippy --all-targets -- -D warnings` and passes `cargo test`, for all db × plugin combinations, via `novastack add`, and after `novastack generate`. Verified on stable and on Rust 1.88 (the pinned MSRV).
- Smoke-tested: sqlite + all plugins runs; `/health`, `/auth/login`, protected `/auth/me` (401 without/with bad token), `/metrics` (repeat scrapes), graceful SIGTERM.
- `novastack generate {module|handler|service|dto|middleware} <name>` works and auto-registers modules/routes.

## How it works
- `templates_shared/` — the base project, rendered for every `new`.
- `templates/<archetype>/` — files an archetype adds (same path as a shared file = replaces it). Files ending in `.jinja` are rendered; all others are copied verbatim (Askama HTML, `.proto`, `build.rs`).
- `src/archetypes/*.rs` — per-archetype patches (deps, dev-deps, module/route/state wiring) through the same anchors plugins use. `templates_shared/` also branches on `{{ template }}` for axum features and the minimal trims. The archetype is recorded in `novastack.toml` (`template`, default `rest`).

| `-t` | Adds |
|---|---|
| `rest` | `src/api/` (utoipa 6 OpenAPI, Swagger UI `/docs`, in-memory `/api/items` CRUD), `tests/api.rs` |
| `htmx` | `src/web/`, Askama `templates/`, HTMX 2 + Tailwind browser CDN, `tests/web.rs` |
| `grpc` | `proto/greeter.proto`, `build.rs` (bundled protoc), `src/grpc/` (Tonic 0.14 + health + reflection, merged into the Axum router; axum `http2`), `tests/greeter.rs` |
| `realtime` | `src/realtime/` (`Hub` in `AppState`, `/ws`, `/events`, `POST /publish`), `static/index.html` demo, `tests/realtime.rs` |
| `minimal` | nothing extra; base with trimmed deps (no CORS/compression/uuid/chrono, reduced tokio features) |
- `templates/plugins/<plugin>/` — files a plugin adds.
- `templates/generate/<kind>/` — component templates for `generate`.
- Plugins and `generate` edit existing files only through **anchor comments** (`// NOVASTACK:ROUTES`, `CONFIG_FIELDS`, `STATE_SETUP`, `STARTUP`, `# NOVASTACK:ENV`, … see `src/scaffold/patcher.rs`). `AnchorPatcher::patch` fails if an anchor is missing. Do not remove anchors from templates.
- Generated projects pin `rust-version` (`GENERATED_RUST_VERSION` in `src/scaffold/engine.rs`) and use `resolver = "3"` so dependency resolution respects it. Bump both together and re-run the matrix on that toolchain.

## Build & test
```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test                                                   # fast suite
cargo test --test generated_compile -- --ignored             # compile matrix, needs network, ~5 min warm
```

## Known gaps / next steps (review roadmap)
3. ~~Archetypes~~ done (`feat/archetypes`). Follow-ups: plugins are HTTP-only (no gRPC auth interceptor / WebSocket auth); example resources are in-memory, not SQLx.
4. **CI:** GitHub Actions for fmt, clippy, test, plus the ignored compile matrix.
5. **Docs/licensing + site:** project site on Cloudflare Pages at **novastack.pages.dev** (Cloudflare only, no GitHub Pages mirror; same setup as dwcode/Control Plane: GitHub Actions deploy on main with CLOUDFLARE_API_TOKEN/ACCOUNT_ID secrets). README claims bundled CI and an OpenTelemetry plugin, links LICENSE-MIT/LICENSE-APACHE which don't exist; not on crates.io so `cargo install novastack` fails.
6. **Polish:**
   - otel: real OTLP trace export (now Prometheus only)
   - validate project names in non-interactive mode (`new ../x` escapes `--out`)
   - real `created_at` timestamp in `novastack.toml` (hardcoded)
   - `add redis` doesn't update docker-compose (prints a hint)
   - the auth login handler is a stub that accepts any non-empty credentials (clearly marked in the generated code)
   - plugin error messages use Debug names (`'Auth'`)
- **Later discussion (user):** deploying NovaStack-generated apps to Cloudflare (would need a workers-rs target).
- **Docker** images were not built in verification (only `cargo` builds); verify `docker build` once Docker is available in CI.
