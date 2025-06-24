# 构建阶段：Alpine + Rust nightly
FROM alpine:3.19 AS builder

# 安装构建依赖和 rustup
RUN apk add --no-cache build-base musl-dev openssl-dev pkgconf curl binutils && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly

# 设置环境变量
ENV PATH="/root/.cargo/bin:${PATH}"

# 创建工作目录并提前下载依赖
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

# 复制代码并编译 release 版本
COPY . .
RUN cargo build --release && strip target/release/quarkdrive-webdav

# 运行阶段：极简，仅需运行依赖
FROM alpine:3.19

# 安装 OpenSSL 运行库
RUN apk add --no-cache libssl3

# 创建用户并设置工作目录
RUN addgroup -S app && adduser -S app -G app
WORKDIR /app

# 复制编译好的二进制文件
COPY --from=builder /app/target/release/quarkdrive-webdav /usr/local/bin/quarkdrive-webdav

# 切换到非 root 用户并设置入口点
USER app
ENTRYPOINT ["/usr/local/bin/quarkdrive-webdav"]