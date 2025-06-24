# 构建阶段
FROM rustlang/rust:nightly as builder

ARG TARGET=x86_64-unknown-linux-musl

WORKDIR /app
COPY . .

RUN apt-get update && \
    apt-get install -y musl-tools && \
    rustup target add ${TARGET} && \
    cargo build --release --target ${TARGET} && \
    strip target/${TARGET}/release/quarkdrive-webdav

# 运行阶段（极简）
FROM alpine:latest
RUN addgroup -S app && adduser -S app -G app
WORKDIR /app
ARG TARGET=x86_64-unknown-linux-musl
COPY --from=builder /app/target/${TARGET}/release/quarkdrive-webdav /usr/local/bin/quarkdrive-webdav
USER app
ENTRYPOINT ["/usr/local/bin/quarkdrive-webdav"]