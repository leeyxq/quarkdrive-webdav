# quarkdrive-webdav
夸克网盘 WebDAV 服务

[![Docker Image](https://img.shields.io/badge/version-latest-blue)](https://ghcr.io/chenqimiao/quarkdrive-webdav)
[![Crates.io](https://img.shields.io/crates/v/quarkdrive-webdav.svg)](https://crates.io/crates/quarkdrive-webdav)


夸克云盘 WebDAV 服务，主要使用场景为配合支持 WebDAV 协议的客户端 App 如 [Infuse](https://firecore.com/infuse)、[nPlayer](https://nplayer.com)
等实现在电视上直接观看云盘视频内容， 支持客户端 App 直接从夸克云盘获取文件播放而不经过运行本应用的服务器中转, 支持上传文件，但受限于 WebDAV 协议不支持文件秒传。


如果项目对你有帮助，欢迎 Star

> **Note**
>
> 本项目作者没有上传需求, 所以暂时还没有开发上传功能，后续考虑迭代

## 安装

可以从 [GitHub Releases](https://github.com/chenqimiao/quarkdrive-webdav/releases) 页面下载预先构建的二进制包


## Docker 运行

```bash
docker run -d --name=quarkdrive-webdav --restart=unless-stopped -p 8080:8080 \
  -e COOKIE='you quark cookie' \
  -e WEBDAV_AUTH_USER=admin \
  -e WEBDAV_AUTH_PASSWORD=admin \
  ghcr.io/chenqimiao/quarkdrive-webdav:latest
```

其中，`COOKIE` 环境变量为你的夸克云盘 `cookie`，`WEBDAV_AUTH_USER`
和 `WEBDAV_AUTH_PASSWORD` 为连接 WebDAV 服务的用户名和密码。



点击 Create (创建)后启动，用webdav客户端连接http://nas地址:8080 即可



#### 本项目参考了以下开源项目，特此鸣谢
- [aliyundrive-webdav](https://github.com/messense/aliyundrive-webdav) 
