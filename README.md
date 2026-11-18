# NovaStack ⚡

> **Zero-friction production-ready Rust web backend scaffolding tool.**

NovaStack (`novastack`) eliminates days of repetitive wiring boilerplate by generating battle-tested, production-ready Rust web backends equipped with **Axum**, **SQLx**, **Tower middleware**, **structured JSON tracing**, and **Docker containerization** out of the box.

---

## Features

- **🚀 Axum 0.8 Engine**: Modular routing, typed extractors, and Tower HTTP middleware (CORS, timeouts, request tracing, compression).
- **🗄️ SQLx 0.8 Connection Pooling**: Pre-configured connection management with embedded migrations and compile-time verified queries (`SQLX_OFFLINE=true`).
- **🛡️ Hardened Error Handling**: Centralized `AppError` implementing Axum's `IntoResponse` that guarantees zero sensitive internal leakage to HTTP clients while logging root causes.
- **📊 Production Observability**: Structured JSON logging powered by `tracing` and `tracing-subscriber`, paired with clean OS signal traps (`SIGINT`/`SIGTERM`) for graceful server drain.
- **🐳 Turnkey Containerization**: Multi-stage `Dockerfile` with `cargo-chef` dependency caching, `docker-compose.yml`, and GitHub Actions CI.
- **🔌 Modular Plugins**: Opt-in features via `novastack add <plugin>`:
  - **JWT Auth**: Session verification, password hashing, user entities.
  - **OpenTelemetry & Prometheus**: Distributed tracing and `/metrics` scrape endpoint.
  - **Redis Cache**: Connection pooling and caching layers with `deadpool-redis`.
  - **Background Workers**: Task queues and background job supervisors.

---

## Installation

```bash
cargo install novastack
```

---

## Quick Start

### 1. Interactive Wizard
```bash
novastack new my-api
```

Follow the prompts to select your database backend, plugins, and metadata.

### 2. Fast CLI Flag Mode
```bash
novastack new my-api --db postgres --plugins auth,otel --no-interactive
```

### 3. Run Your New Backend
```bash
cd my-api
cargo run
```

### 4. Add Plugins to Existing Projects
```bash
novastack add redis
```

---

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).
