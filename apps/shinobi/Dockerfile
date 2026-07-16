# Build stage
FROM rust:1.81-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev curl \
    && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY build.rs ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN mkdir frontend && echo "{}" > frontend/package.json
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src frontend

COPY frontend ./frontend
RUN npm install --prefix frontend --legacy-peer-deps && \
    npx --prefix frontend ng build 2>/dev/null || \
    echo "Angular build skipped (fallback)"

COPY src ./src
COPY build.rs ./
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    chromium \
    --no-install-recommends && \
    rm -rf /var/lib/apt/lists/*

RUN adduser --disabled-password --gecos "" shinobi

WORKDIR /app
COPY --from=builder /app/target/release/shinobi .

COPY static ./static || true

RUN mkdir -p /data/downloads && chown -R shinobi:shinobi /data /app

USER shinobi

ENV PORT=8080
ENV DATA_DIR=/data/downloads

EXPOSE 8080

CMD ["./shinobi"]
