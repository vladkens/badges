FROM --platform=$BUILDPLATFORM rust:alpine AS builder
RUN apk add --no-cache musl-dev openssl-dev zig && \
  rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl && \
  cargo install cargo-zigbuild

WORKDIR /app
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/app/target \
  cargo zigbuild -r --target x86_64-unknown-linux-musl --target aarch64-unknown-linux-musl && \
  mkdir -p /out/linux/ && \
  cp target/x86_64-unknown-linux-musl/release/badges /out/linux/amd64 && \
  cp target/aarch64-unknown-linux-musl/release/badges /out/linux/arm64

FROM alpine:latest AS runtime
LABEL org.opencontainers.image.source="https://github.com/vladkens/badges"
WORKDIR /app

RUN addgroup -S appgroup && adduser -S appuser -G appgroup
USER appuser

ARG TARGETPLATFORM
COPY --from=builder /out/${TARGETPLATFORM} /app/badges

ENV HOST=0.0.0.0 PORT=8080
EXPOSE ${PORT}
HEALTHCHECK CMD wget --no-verbose --tries=1 --spider http://127.0.0.1:${PORT}/health || exit 1

CMD ["/app/badges"]
