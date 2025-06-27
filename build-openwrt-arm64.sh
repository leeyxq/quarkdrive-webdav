#!/bin/bash

# quarkdrive-webdav OpenWrt ARM64 (aarch64_cortex-a53) 编译脚本
# 针对 OpenWrt SNAPSHOT r28532-3deeb7805f 优化
# 目标架构：aarch64_cortex-a53 (mediatek/filogic)

set -e

echo "🚀 开始编译 quarkdrive-webdav OpenWrt ARM64 版本..."
echo "🎯 目标平台: OpenWrt aarch64_cortex-a53 (mediatek/filogic)"

# 检查是否安装了必要的工具
if ! command -v aarch64-linux-musl-gcc &> /dev/null; then
    echo "❌ 错误：未找到 aarch64-linux-musl-gcc"
    echo "请先安装: brew install filosottile/musl-cross/musl-cross"
    exit 1
fi

echo "✅ 工具链检查通过"
echo "📋 编译器版本信息:"
aarch64-linux-musl-gcc --version | head -1

# 检查 Rust 目标是否安装
if ! rustup target list --installed | grep -q "aarch64-unknown-linux-musl"; then
    echo "📦 安装 Rust 编译目标..."
    rustup target add aarch64-unknown-linux-musl
fi

# 清理之前的构建
echo "🧹 清理之前的构建..."
cargo clean

# 设置环境变量解决 OpenSSL 交叉编译问题
echo "🔧 配置编译环境..."
export CC_aarch64_unknown_linux_musl=aarch64-linux-musl-gcc
export AR_aarch64_unknown_linux_musl=aarch64-linux-musl-ar
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-musl-gcc

# 使用 rustls 替代 openssl，避免交叉编译问题
echo "🔐 配置 TLS 库（使用 rustls 避免 OpenSSL 交叉编译问题）..."
# 完全禁用 OpenSSL 相关功能
export OPENSSL_NO_VENDOR=1
unset OPENSSL_LIB_DIR
unset OPENSSL_INCLUDE_DIR
unset OPENSSL_DIR

# OpenWrt 特定优化配置
echo "⚡ 配置 OpenWrt 特定优化..."
export TARGET_CC=aarch64-linux-musl-gcc
export TARGET_CXX=aarch64-linux-musl-g++
export TARGET_AR=aarch64-linux-musl-ar

# 输出关键环境变量
echo "📊 编译环境变量:"
echo "  CC_aarch64_unknown_linux_musl: $CC_aarch64_unknown_linux_musl"
echo "  AR_aarch64_unknown_linux_musl: $AR_aarch64_unknown_linux_musl"
echo "  CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER: $CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER"

# 编译 OpenWrt ARM64 版本
echo "🔨 编译中（针对 Cortex-A53 优化）..."
# 针对 OpenWrt 的优化参数：
# -C target-cpu=cortex-a53: 针对 Cortex-A53 CPU 优化
# -C link-arg=-s: 去除调试符号，减小文件大小
# -C opt-level=z: 优化文件大小（OpenWrt 存储空间有限）
# -C panic=abort: 减小二进制文件大小
export RUSTFLAGS='-C target-cpu=cortex-a53 -C link-arg=-s -C opt-level=z -C panic=abort'
cargo build --release --target aarch64-unknown-linux-musl

# 检查编译结果
if [ -f "target/aarch64-unknown-linux-musl/release/quarkdrive-webdav" ]; then
    echo "✅ 编译成功！"
    echo "📂 二进制文件位置: target/aarch64-unknown-linux-musl/release/quarkdrive-webdav"
    
    # 显示文件信息
    echo "📊 文件信息:"
    file target/aarch64-unknown-linux-musl/release/quarkdrive-webdav
    ls -lh target/aarch64-unknown-linux-musl/release/quarkdrive-webdav
    
    # 使用 UPX 进一步压缩（如果可用）
    if command -v upx &> /dev/null; then
        echo "📦 使用 UPX 压缩二进制文件..."
        cp target/aarch64-unknown-linux-musl/release/quarkdrive-webdav target/aarch64-unknown-linux-musl/release/quarkdrive-webdav.upx
        upx --best target/aarch64-unknown-linux-musl/release/quarkdrive-webdav.upx
        echo "📊 压缩后文件信息:"
        ls -lh target/aarch64-unknown-linux-musl/release/quarkdrive-webdav.upx
    else
        echo "💡 提示：安装 UPX 可以进一步压缩二进制文件大小"
        echo "安装命令: brew install upx"
    fi
    
    # 创建 OpenWrt 分发包
    echo "📦 创建 OpenWrt 分发包..."
    mkdir -p dist-openwrt
    cp target/aarch64-unknown-linux-musl/release/quarkdrive-webdav dist-openwrt/
    
    # 如果有 UPX 压缩版本，也包含进去
    if [ -f "target/aarch64-unknown-linux-musl/release/quarkdrive-webdav.upx" ]; then
        cp target/aarch64-unknown-linux-musl/release/quarkdrive-webdav.upx dist-openwrt/
    fi
    
    cd dist-openwrt
    tar czf ../quarkdrive-webdav-openwrt-aarch64.tar.gz *
    cd ..
    echo "✅ OpenWrt 分发包已创建: quarkdrive-webdav-openwrt-aarch64.tar.gz"
    
    echo ""
    echo "🎯 部署说明："
    echo "1. 将 quarkdrive-webdav-openwrt-aarch64.tar.gz 上传到 OpenWrt 设备"
    echo "2. 解压: tar -xzf quarkdrive-webdav-openwrt-aarch64.tar.gz"
    echo "3. 赋予执行权限: chmod +x quarkdrive-webdav"
    echo "4. 运行: ./quarkdrive-webdav --help"
    echo ""
    echo "💡 针对 OpenWrt 的优化："
    echo "- 使用 Cortex-A53 CPU 特定优化"
    echo "- 最小化文件大小（opt-level=z）"
    echo "- 静态链接，无需额外依赖"
    echo "- 使用 panic=abort 减小二进制大小"
else
    echo "❌ 编译失败"
    exit 1
fi 