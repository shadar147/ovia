# ---- Builder stage ----
FROM rust:1.83-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY crates/common/Cargo.toml crates/common/Cargo.toml
COPY crates/config/Cargo.toml crates/config/Cargo.toml
COPY crates/db/Cargo.toml crates/db/Cargo.toml
COPY crates/matching/Cargo.toml crates/matching/Cargo.toml
COPY services/api/Cargo.toml services/api/Cargo.toml
COPY services/ingest/Cargo.toml services/ingest/Cargo.toml
COPY services/metrics/Cargo.toml services/metrics/Cargo.toml
COPY services/rag/Cargo.toml services/rag/Cargo.toml
COPY services/scheduler/Cargo.toml services/scheduler/Cargo.toml

# Create stub source files so cargo can resolve the workspace
RUN mkdir -p crates/common/src crates/config/src crates/db/src crates/matching/src \
    && echo "fn main(){}" > crates/common/src/lib.rs \
    && echo "fn main(){}" > crates/config/src/lib.rs \
    && echo "fn main(){}" > crates/db/src/lib.rs \
    && echo "fn main(){}" > crates/matching/src/lib.rs \
    && for svc in api ingest metrics rag scheduler; do \
         mkdir -p services/$svc/src && echo "fn main(){}" > services/$svc/src/main.rs; \
       done

# Build dependencies only (cached layer)
RUN cargo build --release --workspace 2>/dev/null || true

# Copy real source code
COPY crates/ crates/
COPY services/ services/

# Touch source files to invalidate cache for real build
RUN find crates services -name "*.rs" -exec touch {} +

# Build all workspace binaries
RUN cargo build --release --workspace

# ---- Runtime stage ----
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/ovia-api /usr/local/bin/
COPY --from=builder /build/target/release/ovia-ingest /usr/local/bin/
COPY --from=builder /build/target/release/ovia-metrics /usr/local/bin/
COPY --from=builder /build/target/release/ovia-rag /usr/local/bin/
COPY --from=builder /build/target/release/ovia-scheduler /usr/local/bin/

RUN useradd --system --no-create-home ovia
USER ovia

EXPOSE 8080

CMD ["ovia-api"]
