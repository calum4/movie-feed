FROM --platform=$BUILDPLATFORM rust:1-alpine3.22 AS chef

WORKDIR /app/

RUN apk add --no-cache musl-dev openssl-dev perl make zig


RUN cargo install --locked cargo-zigbuild cargo-chef
RUN rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl riscv64gc-unknown-linux-musl

FROM --platform=$BUILDPLATFORM chef AS planner

COPY Cargo.lock Cargo.toml ./
COPY lib/ lib/

RUN cargo chef prepare --recipe-path recipe.json

FROM --platform=$BUILDPLATFORM chef AS builder

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json --zigbuild --target x86_64-unknown-linux-musl \
    --target aarch64-unknown-linux-musl --target riscv64gc-unknown-linux-musl

COPY Cargo.lock Cargo.toml ./
COPY lib/ lib/

RUN cargo zigbuild --release --target x86_64-unknown-linux-musl --target aarch64-unknown-linux-musl \
    --target riscv64gc-unknown-linux-musl

RUN mkdir /target && \
    cd target/ && \
    cp --parents */release/movie-feed /target/

FROM alpine:3.22 AS selector

ARG TARGETARCH

COPY --from=builder /target/ /target/

RUN case $TARGETARCH in \
      amd64) \
        target="x86_64-unknown-linux-musl" \
        ;; \
      arm64) \
        target="aarch64-unknown-linux-musl" \
        ;; \
      riscv64) \
        target="riscv64gc-unknown-linux-musl" \
        ;; \
      *) \
        echo "unknown arch" \
        exit 1 \
        ;; \
    esac && \
    mv "/target/$target/release/movie-feed" /movie-feed

FROM alpine:3.22 AS movie-feed

WORKDIR /app

LABEL org.opencontainers.image.source="https://github.com/Calum4/movie-feed"
LABEL org.opencontainers.image.description="Keep up-to-date with your favourite actors upcoming work via an RSS feed!"
LABEL org.opencontainers.image.licenses="MIT OR Apache2"

RUN apk add --no-cache curl

COPY --from=selector /movie-feed movie-feed

HEALTHCHECK --interval=15s --timeout=1s --retries=10 --start-period=15s \
    CMD curl -sSf -o /dev/null "http://${LISTEN_ADDR:-127.0.0.1}:${LISTEN_PORT:-9000}/ok" || exit 1

CMD ["/app/movie-feed"]