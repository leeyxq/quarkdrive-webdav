# 构建阶段：用 Alpine 保证 musl+openssl 兼容
FROM alpine:3.19 as builder

RUN apk add --no-cache build-base musl-dev openssl-dev pkgconf cargo rustup binutils
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