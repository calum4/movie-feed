FROM rust:1-alpine3.22 AS builder

ARG BUILD_ENVIRONMENT
WORKDIR /app/

RUN apk add --no-cache musl-dev openssl-dev perl make

COPY Cargo.lock Cargo.toml ./
COPY lib/ lib/

RUN echo "$BUILD_ENVIRONMENT" > .env && cargo install --path ./lib/movie-feed

FROM alpine:3.22 AS movie-feed

WORKDIR /app

LABEL org.opencontainers.image.source="https://github.com/Calum4/movie-feed"
LABEL org.opencontainers.image.description="Keep up-to-date with your favourite actors upcoming work via an RSS feed!"
LABEL org.opencontainers.image.licenses="MIT OR Apache2"

RUN apk add --no-cache curl

COPY --from=builder /usr/local/cargo/bin/movie-feed movie-feed

HEALTHCHECK --interval=15s --timeout=1s --retries=10 --start-period=15s \
    CMD curl -sSf -o /dev/null "http://${LISTEN_ADDR:-127.0.0.1}:${LISTEN_PORT:-9000}/ok" || exit 1

CMD ["/app/movie-feed"]
