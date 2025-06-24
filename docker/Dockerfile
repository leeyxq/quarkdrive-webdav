# 构建阶段：Alpine + Rust nightly
FROM alpine:3.19 as builder

RUN apk add --no-cache build-base musl-dev openssl-dev pkgconf rustup binutils
RUN rustup toolchain install nightly && rustup default nightly
WORKDIR /app
COPY . .
RUN cargo build --release && strip target/release/quarkdrive-webdav

# 运行阶段：极简
FROM alpine:3.19
RUN addgroup -S app && adduser -S app -G app
WORKDIR /app
COPY --from=builder /app/target/release/quarkdrive-webdav /usr/local/bin/quarkdrive-webdav
USER app
ENTRYPOINT ["/usr/local/bin/quarkdrive-webdav"]