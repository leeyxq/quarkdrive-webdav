#!/bin/sh

# QuarkDrive WebDAV OpenWrt 安装脚本
# 版本: 1.0.9
# 用途: 在 OpenWrt 系统上安装和配置 QuarkDrive WebDAV 服务

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置变量
PROG_NAME="quarkdrive-webdav"
INSTALL_DIR="/usr/bin"
CONFIG_DIR="/etc/config"
INIT_DIR="/etc/init.d"
LOG_DIR="/var/log"

# 源文件路径
SOURCE_BIN="./quarkdrive-webdav"
SOURCE_INIT="./quarkdrive-webdav.init"
SOURCE_CONFIG="./quarkdrive.uci"

# 目标路径
TARGET_BIN="${INSTALL_DIR}/${PROG_NAME}"
TARGET_INIT="${INIT_DIR}/${PROG_NAME}"
TARGET_CONFIG="${CONFIG_DIR}/quarkdrive"

print_info() {
    echo -e "${BLUE}[信息]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[成功]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[警告]${NC} $1"
}

print_error() {
    echo -e "${RED}[错误]${NC} $1"
}

# 检查是否为 root 用户
check_root() {
    if [ "$(id -u)" -ne 0 ]; then
        print_error "请使用 root 用户或 sudo 运行此脚本"
        exit 1
    fi
}

# 检查 OpenWrt 系统
check_openwrt() {
    if [ ! -f "/etc/openwrt_release" ]; then
        print_warning "检测到非 OpenWrt 系统，继续安装可能遇到兼容性问题"
        read -p "是否继续安装? (y/N): " choice
        case "$choice" in 
            y|Y ) print_info "继续安装...";;
            * ) print_info "安装已取消"; exit 0;;
        esac
    else
        print_success "检测到 OpenWrt 系统"
        . /etc/openwrt_release
        print_info "OpenWrt 版本: $DISTRIB_DESCRIPTION"
    fi
}

# 检查源文件
check_source_files() {
    print_info "检查源文件..."
    
    if [ ! -f "$SOURCE_BIN" ]; then
        print_error "找不到可执行文件: $SOURCE_BIN"
        print_info "请确保在正确的目录中运行此脚本，或先编译程序"
        exit 1
    fi
    
    if [ ! -f "$SOURCE_INIT" ]; then
        print_error "找不到启动脚本: $SOURCE_INIT"
        exit 1
    fi
    
    if [ ! -f "$SOURCE_CONFIG" ]; then
        print_error "找不到配置文件: $SOURCE_CONFIG"
        exit 1
    fi
    
    print_success "所有源文件检查完毕"
}

# 停止现有服务
stop_existing_service() {
    if [ -f "$TARGET_INIT" ]; then
        print_info "停止现有服务..."
        "$TARGET_INIT" stop 2>/dev/null || true
        sleep 2
    fi
}

# 安装二进制文件
install_binary() {
    print_info "安装二进制文件到 $TARGET_BIN"
    
    # 备份现有文件
    if [ -f "$TARGET_BIN" ]; then
        print_info "备份现有二进制文件"
        cp "$TARGET_BIN" "${TARGET_BIN}.backup.$(date +%Y%m%d_%H%M%S)"
    fi
    
    # 安装新文件
    cp "$SOURCE_BIN" "$TARGET_BIN"
    chmod 755 "$TARGET_BIN"
    
    print_success "二进制文件安装完成"
}

# 安装启动脚本
install_init_script() {
    print_info "安装启动脚本到 $TARGET_INIT"
    
    # 备份现有脚本
    if [ -f "$TARGET_INIT" ]; then
        print_info "备份现有启动脚本"
        cp "$TARGET_INIT" "${TARGET_INIT}.backup.$(date +%Y%m%d_%H%M%S)"
    fi
    
    # 安装新脚本
    cp "$SOURCE_INIT" "$TARGET_INIT"
    chmod 755 "$TARGET_INIT"
    
    print_success "启动脚本安装完成"
}

# 安装配置文件
install_config() {
    print_info "安装配置文件到 $TARGET_CONFIG"
    
    # 如果配置文件已存在，询问是否覆盖
    if [ -f "$TARGET_CONFIG" ]; then
        print_warning "配置文件已存在"
        read -p "是否覆盖现有配置? (y/N): " choice
        case "$choice" in 
            y|Y ) 
                print_info "备份现有配置文件"
                cp "$TARGET_CONFIG" "${TARGET_CONFIG}.backup.$(date +%Y%m%d_%H%M%S)"
                cp "$SOURCE_CONFIG" "$TARGET_CONFIG"
                print_success "配置文件已更新"
                ;;
            * ) 
                print_info "保留现有配置文件"
                ;;
        esac
    else
        cp "$SOURCE_CONFIG" "$TARGET_CONFIG"
        print_success "配置文件安装完成"
    fi
}

# 创建日志目录
create_log_dir() {
    print_info "创建日志目录"
    mkdir -p "$LOG_DIR"
    print_success "日志目录创建完成"
}

# 启用开机自启
enable_autostart() {
    print_info "配置开机自启动"
    
    # 启用服务
    "$TARGET_INIT" enable 2>/dev/null || {
        print_warning "无法使用 enable 命令，手动创建软链接"
        # 手动创建软链接
        ln -sf "$TARGET_INIT" "/etc/rc.d/S99${PROG_NAME}" 2>/dev/null || true
        ln -sf "$TARGET_INIT" "/etc/rc.d/K10${PROG_NAME}" 2>/dev/null || true
    }
    
    print_success "开机自启动已配置"
}

# 启动服务
start_service() {
    print_info "启动 QuarkDrive WebDAV 服务"
    
    if "$TARGET_INIT" start; then
        print_success "服务启动成功"
        sleep 2
        "$TARGET_INIT" status
    else
        print_error "服务启动失败"
        print_info "请检查配置文件和日志: $TARGET_INIT logs"
        return 1
    fi
}

# 显示安装结果
show_result() {
    echo ""
    echo "=============================================="
    echo -e "${GREEN}QuarkDrive WebDAV 安装完成!${NC}"
    echo "=============================================="
    echo "可执行文件: $TARGET_BIN"
    echo "启动脚本: $TARGET_INIT" 
    echo "配置文件: $TARGET_CONFIG"
    echo "日志目录: $LOG_DIR"
    echo ""
    echo "常用命令:"
    echo "  启动服务: $TARGET_INIT start"
    echo "  停止服务: $TARGET_INIT stop"
    echo "  重启服务: $TARGET_INIT restart"
    echo "  查看状态: $TARGET_INIT status"
    echo "  查看日志: $TARGET_INIT logs"
    echo "  服务信息: $TARGET_INIT info"
    echo ""
    echo "配置文件: $TARGET_CONFIG"
    echo "请根据需要修改配置后重启服务"
    echo "=============================================="
}

# 卸载函数
uninstall() {
    print_info "开始卸载 QuarkDrive WebDAV"
    
    # 停止服务
    if [ -f "$TARGET_INIT" ]; then
        print_info "停止服务"
        "$TARGET_INIT" stop 2>/dev/null || true
        
        # 禁用自启动
        print_info "禁用自启动"
        "$TARGET_INIT" disable 2>/dev/null || {
            rm -f "/etc/rc.d/S99${PROG_NAME}" 2>/dev/null || true
            rm -f "/etc/rc.d/K10${PROG_NAME}" 2>/dev/null || true
        }
    fi
    
    # 删除文件
    print_info "删除文件"
    rm -f "$TARGET_BIN"
    rm -f "$TARGET_INIT"
    
    # 询问是否删除配置和日志
    read -p "是否删除配置文件和日志? (y/N): " choice
    case "$choice" in 
        y|Y ) 
            rm -f "$TARGET_CONFIG"
            rm -f "/var/log/quarkdrive-webdav.log"
            rm -f "/var/log/quarkdrive-webdav.err"
            rm -f "/var/run/quarkdrive-webdav.pid"
            print_info "配置文件和日志已删除"
            ;;
        * ) 
            print_info "保留配置文件和日志"
            ;;
    esac
    
    print_success "QuarkDrive WebDAV 卸载完成"
}

# 主函数
main() {
    case "${1:-install}" in
        install)
            echo "=============================================="
            echo "QuarkDrive WebDAV OpenWrt 安装程序"
            echo "版本: 1.0.9"
            echo "=============================================="
            
            check_root
            check_openwrt
            check_source_files
            stop_existing_service
            install_binary
            install_init_script
            install_config
            create_log_dir
            enable_autostart
            start_service
            show_result
            ;;
        uninstall)
            check_root
            uninstall
            ;;
        *)
            echo "用法: $0 [install|uninstall]"
            echo ""
            echo "命令说明:"
            echo "  install   - 安装 QuarkDrive WebDAV 服务 (默认)"
            echo "  uninstall - 卸载 QuarkDrive WebDAV 服务"
            exit 1
            ;;
    esac
}

# 信号处理
trap 'print_error "安装被中断"; exit 1' INT TERM

# 执行主函数
main "$@" 