# AGENTS.md

Rust Cargo workspace backend (Axum + SeaORM) split into clean-architecture layers.
The Figma element-fetch feature is implemented; `migration` is still a stub.

## Layout and dependency direction

- `domain/` — entities/value objects plus ports. `figma::model` parses Figma links, models documents/nodes, page layouts and bounding boxes; `figma::repository` and `figma::usecase` are traits. No I/O.
- `usecase/` — application logic. `FigmaService` implements `domain::figma::usecase::FigmaUseCase` and depends only on `domain`. It resolves the page referenced by a link, fetches its full subtree, collects top-level render units (frames, sections, groups, components, etc.) and all frames (including frames nested inside sections/groups), then computes the combined bounding box.
- `infra/` — adapters. `figma::FigmaClient` implements `FigmaRepository` with `reqwest`; `mappers/figma.rs` converts API DTOs to domain models. `FigmaClient::render_elements` downloads each element render and composes the page PNG with the `image` crate. Also holds the SeaORM persistence (`sqlx-postgres`).
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
- Page endpoints resolve the page from `node-id` (a page id, a top-level frame id, or a nested node id via a full-file fallback); without `node-id` they use the first page. They fetch the full page subtree so section contents are available. `elements` contains the top-level render units (frames, sections, groups, components, etc.; `SLICE` is excluded and invisible/zero-opacity nodes are skipped). `frames` contains every frame on the page, including frames nested inside sections/groups.
- `render_elements` composes the top-level render units at their absolute positions. It requests one Figma image per element (batches of 100), then stitches them into one PNG. Output is capped at 32 megapixels; lower `scale` for large pages.

## Commands

All crates use Rust edition 2024 / resolver 3.

- Check everything: `cargo check --workspace --all-targets` (requires the toolchain update above)
- Check without `migration`: `cargo check -p api -p domain -p infra -p usecase --all-targets`
- Run server: `cargo run -p api` (Axum on `HOST`/`PORT`, default `127.0.0.1:8000`)
- Fetch an element: `curl "http://127.0.0.1:8000/api/figma/element?url=<url-encoded-figma-link>"`
- Inspect a page layout/bounding box: `curl "http://127.0.0.1:8000/api/figma/page?url=<url-encoded-figma-link>"`
- Render a page PNG: `curl "http://127.0.0.1:8000/api/figma/page/render?url=<url-encoded-figma-link>&scale=1" -o page.png`
- Test without `migration`: `cargo test -p domain -p usecase -p infra -p api`
- Single feature test: `cargo test -p domain figma`
- Format: `cargo fmt --all`
- Lint: `cargo clippy -p api -p domain -p infra -p usecase --all-targets`

No custom rustfmt, clippy, CI, or task-runner config exists; use the standard Cargo commands.
