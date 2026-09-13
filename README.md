# EPSX Analytics Platform

EPSX is a Rust workspace for a Dioxus analytics frontend, an admin portal, Axum microservices, and Foundry contracts. The repository has no application-owned browser scripting toolchain; wasm-bindgen glue is generated under `target/` during a build and is never committed.

## Quick start

Install stable Rust, the `wasm32-unknown-unknown` target, `wasm-bindgen-cli` 0.2.123, Docker, and Foundry. Then configure `.env` from `.env.example` and run:

```bash
cargo xtask env validate
cargo xtask dev --all
```

| Service | Local URL | Implementation |
|---|---|---|
| Frontend | http://localhost:3000 | Dioxus SSR + Rust/WASM |
| Admin | http://localhost:3001 | Dioxus SSR + Rust/WASM |
| Backend | http://localhost:8080 | Rust + Axum |

## Workspace

```text
apps/                  Dioxus frontends and Rust application binaries
services/              Rust microservices
shared/rust/           shared Rust domain, UI, client, and infrastructure crates
apps/contracts/        Foundry contracts and pinned submodules
e2e/                   Rust WebDriver scenario contract and fixtures
xtask/                 workspace development, build, audit, and E2E commands
```

Business rules for permissions, plans, ranking, subscriptions, and feature access live in Rust backend crates. Dioxus applications are presentation and interaction layers.

## Commands

```bash
cargo xtask dev --all
cargo xtask build --profile development
cargo xtask build --profile production
cargo xtask test --all
cargo xtask e2e doctor
cargo xtask e2e run --group 0 --browser chromium
cargo xtask e2e report
cargo xtask e2e verify-artifacts
cargo xtask env validate
cargo xtask setup-local
cargo xtask anvil-proxy
cargo xtask assets verify
cargo xtask audit no-node --strict
```

Use `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --workspace --locked` as the local merge gates. Foundry contracts use `forge build --root apps/contracts` and `forge test --root apps/contracts`.

## Environment and deployment

Environment loading is layered through `.env`, `.env.<environment>`, `.env.local`, and `.env.<environment>.local`. Select the environment with `DEPLOYMENT_ENV`, `APP_ENV`, `ENV`, `EPSX_ENV`, or `RUST_ENV`.

Production uses Colima Kubernetes and Cloudflare Tunnel. Production deployment always requires an explicit user instruction; repository changes and migration verification do not authorize a deployment.
