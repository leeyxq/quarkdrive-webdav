# 构建阶段：Alpine + Rust nightly
FROM alpine:3.19 AS builder

# 安装构建依赖和 rustup
RUN apk add --no-cache build-base musl-dev openssl-dev pkgconf curl binutils

# 安装 Rust nightly
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app
COPY . .

# 编译 release 版本并瘦身
RUN cargo build --release && strip target/release/quarkdrive-webdav

# 运行阶段：极简，仅需运行依赖
FROM alpine:3.19

# 安装 OpenSSL 运行库
RUN apk add --no-cache libssl3

RUN addgroup -S app && adduser -S app -G app
WORKDIR /app

COPY --from=builder /app/target/release/quarkdrive-webdav /usr/local/bin/quarkdrive-webdav

USER app
ENTRYPOINT ["/usr/local/bin/quarkdrive-webdav"]