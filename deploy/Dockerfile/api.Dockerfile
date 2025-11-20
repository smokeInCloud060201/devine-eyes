ARG RUST_VERSION=1.90.0
ARG APP_NAME=eyes-devine-server

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
# Build context should be workspace root (.), not ./backend
COPY backend/Cargo.toml ./
COPY Cargo.lock* ./
COPY backend/shared/Cargo.toml ./shared/
COPY backend/services/Cargo.toml ./services/
COPY backend/server/Cargo.toml ./server/

RUN mkdir -p shared/src services/src server/src && \
    echo "fn main() {}" > server/src/main.rs && \
    echo "pub mod lib;" > shared/src/lib.rs && \
    echo "pub mod models;" > shared/src/models.rs && \
    echo "pub mod lib;" > services/src/lib.rs && \
    touch shared/src/models.rs services/src/lib.rs


RUN cargo build --release --bin ${APP_NAME} || true

RUN rm -rf server/src shared/src services/src

COPY backend/shared/ ./shared/
COPY backend/services/ ./services/
COPY backend/server/ ./server/

RUN cargo build --release --bin ${APP_NAME} && \
    cp target/release/${APP_NAME} /bin/server

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

COPY --from=build /bin/server /usr/local/bin/eyes-devine-server

RUN chown appuser:appuser /usr/local/bin/eyes-devine-server && \
    chmod +x /usr/local/bin/eyes-devine-server

USER appuser

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/stats/total || exit 1

CMD ["eyes-devine-server"]

