FROM rust:slim-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY api api
COPY domain domain
COPY infra infra
COPY usecase usecase
COPY migration migration

RUN cargo build --release -p api --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --uid 10001 app
USER app
WORKDIR /app

COPY --from=builder /app/target/release/api /usr/local/bin/api

ENV HOST=0.0.0.0 \
    PORT=8000

EXPOSE 8000

CMD ["api"]
