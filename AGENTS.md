# AGENTS.md

Rust Cargo workspace backend (Axum + SeaORM) split into clean-architecture layers.
The Figma element-fetch feature is implemented; `migration` is still a stub.

## Layout and dependency direction

- `domain/` — entities/value objects plus ports. `figma::model` parses Figma links and models documents/nodes; `figma::repository` and `figma::usecase` are traits. No I/O.
- `usecase/` — application logic. `FigmaService` implements `domain::figma::usecase::FigmaUsecase` and depends only on `domain`.
- `infra/` — adapters. `figma::FigmaClient` implements `FigmaRepository` with `reqwest`; `mappers/figma.rs` converts API DTOs to domain models. Also holds the SeaORM persistence (`sqlx-postgres`).
- `api/` — Axum HTTP server binary and composition root. Depends on `usecase`, `domain`, and `infra`. Real entrypoint: `api/src/main.rs`.
- `migration/` — `sea-orm-migration` crate. Not yet wired into `api` or `infra`, and has no migration files or runner binary.

Keep dependencies pointing inward: `api -> usecase -> domain` and `api -> infra -> domain`.
Do not let `domain` or `usecase` import `api`, `infra`, Axum, SeaORM, or `reqwest`.

## Gotchas

- The root `src/main.rs` is **orphaned**: the root `Cargo.toml` is a virtual workspace with no `[package]`, so this file is never compiled. Put server code in `api/src/main.rs`.
- `migration` pulls `sea-orm 2.0.4`, which requires rustc >= 1.94, but the default toolchain here is 1.93. `cargo check/test --workspace` currently fails with "rustc 1.93.0 is not supported". Run `rustup update stable` first. Until then, target the other crates explicitly.
- `infra` uses SeaORM 1.1.x while `migration` uses 2.0.x (both appear in `Cargo.lock`). Align these versions before writing entities/migrations.
- No README, CI, or database config exists yet. `.env` is gitignored and loaded by `dotenvy`; copy `.env.example` and set `FIGMA_TOKEN` (a Figma personal access token).
- Figma links are accepted in `file`, `design`, `proto`, `board`, and `slides` forms. URL `node-id=12-34` is normalized to API form `12:34`; a link without `node-id` fetches the whole file.

## Commands

All crates use Rust edition 2024 / resolver 3.

- Check everything: `cargo check --workspace --all-targets` (requires the toolchain update above)
- Check without `migration`: `cargo check -p api -p domain -p infra -p usecase --all-targets`
- Run server: `cargo run -p api` (Axum on `HOST`/`PORT`, default `127.0.0.1:8000`)
- Fetch an element: `curl "http://127.0.0.1:8000/api/figma/element?url=<url-encoded-figma-link>"`
- Test without `migration`: `cargo test -p domain -p usecase -p infra -p api`
- Single feature test: `cargo test -p domain figma`
- Format: `cargo fmt --all`
- Lint: `cargo clippy -p api -p domain -p infra -p usecase --all-targets`

No custom rustfmt, clippy, CI, or task-runner config exists; use the standard Cargo commands.
