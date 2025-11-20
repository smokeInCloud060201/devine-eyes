ARG RUST_VERSION=1.90.0
ARG APP_NAME=eyes-devine-worker

FROM rust:${RUST_VERSION}-slim AS build

ARG APP_NAME

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    libpcap-dev \
    clang \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace root files
# Build context should be workspace root (.), not ./worker
# Worker expects ../backend/services and ../backend/shared, so preserve backend/ structure
# backend/services and backend/shared are part of a workspace, so we need the workspace root Cargo.toml
COPY worker/Cargo.toml ./worker/
COPY worker/Cargo.lock* ./worker/
COPY backend/Cargo.toml ./backend/
COPY backend/shared/Cargo.toml ./backend/shared/
COPY backend/services/Cargo.toml ./backend/services/

RUN mkdir -p backend/shared/src backend/services/src worker/src && \
    echo "fn main() {}" > worker/src/main.rs && \
    echo "pub mod lib;" > backend/shared/src/lib.rs && \
    echo "pub mod models;" > backend/shared/src/models.rs && \
    echo "pub mod lib;" > backend/services/src/lib.rs && \
    touch backend/shared/src/models.rs backend/services/src/lib.rs

RUN cd worker && cargo build --release --features network-capture --bin ${APP_NAME} || true

RUN rm -rf worker/src backend/shared/src backend/services/src

COPY backend/shared/ ./backend/shared/
COPY backend/services/ ./backend/services/
COPY worker/ ./worker/

RUN cd worker && cargo build --release --features network-capture --bin ${APP_NAME} && \
    cp /app/worker/target/release/${APP_NAME} /bin/worker

FROM debian:bookworm-slim AS runtime

ARG UID=1000
ARG GID=1000

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    libpcap0.8 \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -g ${GID} appuser && \
    useradd -u ${UID} -g ${GID} -m -s /bin/bash appuser

RUN mkdir -p /var/run/docker.sock.d && \
    chown -R appuser:appuser /var/run/docker.sock.d

WORKDIR /app

COPY --from=build /bin/worker /usr/local/bin/eyes-devine-worker

RUN chown appuser:appuser /usr/local/bin/eyes-devine-worker && \
    chmod +x /usr/local/bin/eyes-devine-worker

USER appuser

EXPOSE 8081

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8081/health || exit 1

CMD ["eyes-devine-worker"]
