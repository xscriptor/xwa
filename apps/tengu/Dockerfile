# =============================================================================
# Stage 1: Frontend (Angular static build)
# =============================================================================
FROM node:20-slim AS frontend-builder

WORKDIR /app/frontend

COPY frontend/package.json frontend/package-lock.json* ./
RUN npm ci --legacy-peer-deps || npm install --legacy-peer-deps

COPY frontend/ .
RUN npx ng build --output-path=../static --configuration=production 2>/dev/null \
    || npx ng build --output-path=../static || echo "Frontend build skipped"

# =============================================================================
# Stage 2: Backend (Rust)
# =============================================================================
FROM rust:1.81-slim-bookworm AS backend-builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Build dependencies first (caching layer)
COPY Cargo.toml Cargo.lock* build.rs ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    mkdir -p src/auditor && \
    echo "pub fn run_audit() {}" > src/auditor/mod.rs && \
    echo "pub mod auditor;" > src/main.rs && \
    cargo build --release 2>/dev/null || true
RUN rm -rf src

# Full build
COPY src ./src
RUN cargo build --release

# =============================================================================
# Stage 3: Runtime (minimal)
# =============================================================================
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN adduser --disabled-password --gecos "" tengu

WORKDIR /app

COPY --from=backend-builder /app/target/release/tengu .
COPY --from=frontend-builder /app/static ./static

RUN mkdir -p /data && chown -R tengu:tengu /data /app

USER tengu

ENV PORT=8080
EXPOSE 8080

CMD ["./tengu"]
