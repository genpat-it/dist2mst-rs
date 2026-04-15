# Stage 1: Build
FROM rust:1.85-slim AS builder

WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY src/ src/

ENV RUSTFLAGS="-C target-cpu=x86-64-v2"
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

COPY --from=builder /build/target/release/dist2mst /usr/local/bin/dist2mst

LABEL org.opencontainers.image.title="dist2mst" \
      org.opencontainers.image.description="Ultra-fast Minimum Spanning Tree construction from distance matrices" \
      org.opencontainers.image.source="https://github.com/genpat-it/dist2mst-rs" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.version="0.1.0" \
      org.opencontainers.image.authors="GenPat <genpat@izs.it>" \
      maintainer="GenPat <genpat@izs.it>"

ENTRYPOINT ["dist2mst"]
CMD ["--help"]
