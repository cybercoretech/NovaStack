# AGENT_HANDOFF.md

## Project
- **Name:** NovaStack (`novastack`), crate lib name `novastack_core`
- **Purpose:** CLI that scaffolds Rust web backends (Axum 0.8, optional SQLx Postgres/SQLite) with optional plugins (auth, otel, redis, workers) and generates components inside existing projects.
- **Repository:** https://github.com/cybercoretech/NovaStack (local: `~/Devspace/Cybercore/tooling/novastack`)
- **Workflow:** always branch + PR; never stack PRs; check CI before merging.

## Current state (2026-11-18)
Branch `fix/scaffold-compiles` (steps 1–2 of the review roadmap):
- Every generated project compiles clean under `cargo clippy --all-targets -- -D warnings` and passes `cargo test`, for all db × plugin combinations, via `novastack add`, and after `novastack generate`. Verified on stable and on Rust 1.88 (the pinned MSRV).
- Smoke-tested: sqlite + all plugins runs; `/health`, `/auth/login`, protected `/auth/me` (401 without/with bad token), `/metrics` (repeat scrapes), graceful SIGTERM.
- `novastack generate {module|handler|service|dto|middleware} <name>` works and auto-registers modules/routes.

## How it works
- `templates_shared/` — the base project, rendered for every `new`.
- `templates/<archetype>/` — optional overlay per `-t` archetype (files here replace shared files with the same path). **Currently empty**: the five archetypes were identical copies of the base and were removed. `-t` is accepted but has no effect yet.
- `templates/plugins/<plugin>/` — files a plugin adds.
- `templates/generate/<kind>/` — component templates for `generate`.
- Plugins and `generate` edit existing files only through **anchor comments** (`// NOVASTACK:ROUTES`, `CONFIG_FIELDS`, `STATE_SETUP`, `STARTUP`, `# NOVASTACK:ENV`, … see `src/scaffold/patcher.rs`). `AnchorPatcher::patch` fails if an anchor is missing. Do not remove anchors from templates.
- Generated projects pin `rust-version` (`GENERATED_RUST_VERSION` in `src/scaffold/engine.rs`) and use `resolver = "3"` so dependency resolution respects it. Bump both together and re-run the matrix on that toolchain.

## Build & test
```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test                                                   # fast suite
cargo test --test generated_compile -- --ignored             # compile matrix, needs network, ~3 min cold
```

## Known gaps / next steps (review roadmap)
3. **Archetypes:** build real `rest` (utoipa/Swagger), `htmx` (Askama), `grpc` (Tonic), `realtime` (WS/SSE) overlays, or trim the advertised list. README and wizard still advertise them.
4. **CI:** GitHub Actions for fmt, clippy, test, plus the ignored compile matrix.
5. **Docs/licensing:** README repeats the "Build Templates" section 4×, claims bundled CI, links LICENSE-MIT/LICENSE-APACHE which don't exist; not on crates.io so `cargo install novastack` fails.
6. **Polish:**
   - otel: real OTLP trace export (now Prometheus only)
   - validate project names in non-interactive mode (`new ../x` escapes `--out`)
   - real `created_at` timestamp in `novastack.toml` (hardcoded)
   - `add redis` doesn't update docker-compose (prints a hint)
   - the auth login handler is a stub that accepts any non-empty credentials (clearly marked in the generated code)
   - plugin error messages use Debug names (`'Auth'`)
- **Docker** images were not built in verification (only `cargo` builds); verify `docker build` once Docker is available in CI.
