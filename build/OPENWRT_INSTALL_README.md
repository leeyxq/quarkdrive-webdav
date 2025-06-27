# QuarkDrive WebDAV OpenWrt 安装指南

## 概述

本指南将帮助您在 OpenWrt 系统上安装和配置 QuarkDrive WebDAV 服务，实现开机自启动。

## 系统要求

- OpenWrt 路由器或设备
- 支持的 CPU 架构 (ARM64/AARCH64)
- 至少 50MB 可用存储空间
- Root 权限访问

## 快速安装

### 1. 下载文件

将以下文件上传到您的 OpenWrt 设备：
- `quarkdrive-webdav` - 主程序可执行文件
- `quarkdrive-webdav.init` - OpenWrt 启动脚本
- `quarkdrive.uci` - UCI 配置文件模板
- `install-openwrt.sh` - 自动安装脚本

### 2. 执行安装

```bash
# 给安装脚本执行权限
chmod +x install-openwrt.sh

# 运行安装脚本
./install-openwrt.sh install
```

### 3. 验证安装

```bash
# 检查服务状态
/etc/init.d/quarkdrive-webdav status

# 查看服务信息
/etc/init.d/quarkdrive-webdav info
```

## 手动安装

如果自动安装脚本遇到问题，可以手动安装：

### 1. 安装二进制文件

```bash
# 复制可执行文件
cp quarkdrive-webdav /usr/bin/
chmod 755 /usr/bin/quarkdrive-webdav
```

### 2. 安装启动脚本

```bash
# 复制启动脚本
cp quarkdrive-webdav.init /etc/init.d/quarkdrive-webdav
chmod 755 /etc/init.d/quarkdrive-webdav
```

### 3. 安装配置文件

```bash
# 复制配置文件
cp quarkdrive.uci /etc/config/quarkdrive
```

### 4. 启用开机自启

```bash
# 启用服务
/etc/init.d/quarkdrive-webdav enable

# 启动服务
/etc/init.d/quarkdrive-webdav start
```

## 配置说明

### UCI 配置文件

配置文件位于 `/etc/config/quarkdrive`，包含以下选项：

```
config config 'config'
    option cookie '夸克网盘的 Cookie 信息'
    option username '夸克网盘用户名'
    option password '夸克网盘密码'
    option port '监听端口，默认 8000'
    option enabled '是否启用服务，1=启用，0=禁用'
```

### 修改配置

您可以通过以下方式修改配置：

#### 方法 1: 直接编辑配置文件

```bash
# 使用 vi 编辑器
vi /etc/config/quarkdrive

# 或使用 nano 编辑器
nano /etc/config/quarkdrive
```

#### 方法 2: 使用 UCI 命令

```bash
# 修改用户名
uci set quarkdrive.config.username='your_username'

# 修改密码
uci set quarkdrive.config.password='your_password'

# 修改端口
uci set quarkdrive.config.port='8080'

# 提交更改
uci commit quarkdrive

# 重启服务
/etc/init.d/quarkdrive-webdav restart
```

### Cookie 获取方法

1. 使用浏览器登录夸克网盘
2. 按 F12 打开开发者工具
3. 转到 Network 标签页
4. 刷新页面
5. 找到任意一个请求，查看 Cookie 头部
6. 复制完整的 Cookie 字符串

## 服务管理

### 基本命令

```bash
# 启动服务
/etc/init.d/quarkdrive-webdav start

# 停止服务
/etc/init.d/quarkdrive-webdav stop

# 重启服务
/etc/init.d/quarkdrive-webdav restart

# 重新加载配置
/etc/init.d/quarkdrive-webdav reload

# 查看服务状态
/etc/init.d/quarkdrive-webdav status

# 查看服务信息
/etc/init.d/quarkdrive-webdav info

# 查看最近日志 (默认20行)
/etc/init.d/quarkdrive-webdav logs

# 查看最近50行日志
/etc/init.d/quarkdrive-webdav logs 50
```

### 开机自启管理

```bash
# 启用开机自启
/etc/init.d/quarkdrive-webdav enable

# 禁用开机自启
/etc/init.d/quarkdrive-webdav disable
```

## 日志管理

### 日志文件位置

- 主日志: `/var/log/quarkdrive-webdav.log`
- 错误日志: `/var/log/quarkdrive-webdav.err`
- PID 文件: `/var/run/quarkdrive-webdav.pid`

### 查看日志

```bash
# 查看主日志
tail -f /var/log/quarkdrive-webdav.log

# 查看错误日志
tail -f /var/log/quarkdrive-webdav.err

# 清空日志
> /var/log/quarkdrive-webdav.log
> /var/log/quarkdrive-webdav.err
```

## 使用 WebDAV

服务启动后，您可以通过以下方式访问 WebDAV：

### WebDAV 地址

```
http://路由器IP:端口号/
例如: http://192.168.1.1:8000/
```

### 客户端配置

#### Windows 资源管理器

1. 打开"此电脑"
2. 右键点击空白处，选择"添加网络位置"
3. 输入 WebDAV 地址
4. 输入夸克网盘的用户名和密码

#### macOS Finder

1. 打开 Finder
2. 按 Cmd+K 或选择"前往" > "连接服务器"
3. 输入 WebDAV 地址
4. 输入夸克网盘的用户名和密码

#### Linux

```bash
# 使用 davfs2 挂载
sudo mount -t davfs http://192.168.1.1:8000/ /mnt/quarkdrive
```

## 故障排除

### 服务无法启动

1. 检查配置文件是否正确：
   ```bash
   /etc/init.d/quarkdrive-webdav info
   ```

2. 查看错误日志：
   ```bash
   /etc/init.d/quarkdrive-webdav logs
   ```

3. 验证二进制文件权限：
   ```bash
   ls -la /usr/bin/quarkdrive-webdav
   ```

### 无法访问 WebDAV

1. 检查端口是否被占用：
   ```bash
   netstat -ln | grep :8000
   ```

2. 检查防火墙设置：
   ```bash
   iptables -L | grep 8000
   ```

3. 验证服务是否正在运行：
   ```bash
   /etc/init.d/quarkdrive-webdav status
   ```

### Cookie 过期

如果遇到认证失败，可能是 Cookie 过期：

1. 重新获取 Cookie
2. 更新配置文件
3. 重启服务

## 升级服务

### 使用安装脚本升级

```bash
# 下载新版本文件后，直接运行安装脚本
./install-openwrt.sh install
```

安装脚本会自动备份现有文件和配置。

### 手动升级

```bash
# 停止服务
/etc/init.d/quarkdrive-webdav stop

# 备份现有二进制文件
cp /usr/bin/quarkdrive-webdav /usr/bin/quarkdrive-webdav.backup

# 复制新版本
cp quarkdrive-webdav /usr/bin/
chmod 755 /usr/bin/quarkdrive-webdav

# 启动服务
/etc/init.d/quarkdrive-webdav start
```

## 卸载服务

### 使用安装脚本卸载

```bash
./install-openwrt.sh uninstall
```

### 手动卸载

```bash
# 停止并禁用服务
/etc/init.d/quarkdrive-webdav stop
/etc/init.d/quarkdrive-webdav disable

# 删除文件
rm -f /usr/bin/quarkdrive-webdav
rm -f /etc/init.d/quarkdrive-webdav
rm -f /etc/config/quarkdrive

# 清理日志 (可选)
rm -f /var/log/quarkdrive-webdav.*
rm -f /var/run/quarkdrive-webdav.pid
```

## 高级配置

### 自定义端口

如果默认端口被占用，可以修改为其他端口：

```bash
# 修改端口为 8080
uci set quarkdrive.config.port='8080'
uci commit quarkdrive
/etc/init.d/quarkdrive-webdav restart
```

### 性能优化

1. **内存限制**: OpenWrt 设备内存有限，建议监控内存使用情况
2. **存储空间**: 定期清理日志文件以节省存储空间
3. **网络优化**: 确保网络连接稳定，特别是上传到夸克网盘时

### 开机延迟启动

如果需要延迟启动服务（例如等待网络连接稳定）：

1. 修改启动脚本的 START 值（更大的数值表示更晚启动）
2. 或在启动脚本中添加 sleep 延迟

## 技术支持

如果您遇到问题：

1. 首先查看日志文件获取错误信息
2. 检查配置文件是否正确
3. 确认网络连接正常
4. 验证夸克网盘账号和 Cookie 有效性

## 许可证

本软件遵循 MIT 许可证，详见 LICENSE 文件。 