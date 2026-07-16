FROM rust:1.96-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./

# Cache dependency compilation
RUN mkdir src && echo 'fn main() {}' > src/main.rs && \
    cargo build --release 2>/dev/null; \
    rm -rf src

COPY src/ src/

# touch forces rustc to rebuild — avoids stale binary from cached layer
RUN touch src/*.rs src/*/*.rs && \
    cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    nmap ca-certificates libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/samurai-tui /usr/local/bin/samurai-tui

ENV SAMURAI_DB_BACKEND=sqlite
ENV SAMURAI_SQLITE_PATH=/data/samurai.db

RUN mkdir -p /data
VOLUME ["/data"]

WORKDIR /data
ENTRYPOINT ["samurai-tui"]
