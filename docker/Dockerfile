# 使用官方 Rust 镜像进行构建
FROM rustlang/rust:nightly as builder

WORKDIR /app
COPY . .

# 构建 release 版本
RUN cargo build --release

# 使用更小的基础镜像运行
FROM debian:bullseye-slim

WORKDIR /app
COPY --from=builder /app/target/release/quarkdrive-webdav /usr/local/bin/quarkdrive-webdav

# 可选：复制静态文件或配置
# COPY static/ /app/static/

# 运行时可通过环境变量传递参数
ENTRYPOINT ["/usr/local/bin/quarkdrive-webdav"]