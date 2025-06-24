# quarkdrive-webdav
夸克网盘 WebDAV 服务

[![Docker Image](https://img.shields.io/badge/version-latest-blue)](https://ghcr.io/chenqimiao/quarkdrive-webdav)
[![Crates.io](https://img.shields.io/crates/v/quarkdrive-webdav.svg)](https://crates.io/crates/quarkdrive-webdav)


夸克云盘 WebDAV 服务，主要使用场景为配合支持 WebDAV 协议的客户端 App 如 [Infuse](https://firecore.com/infuse)、[nPlayer](https://nplayer.com)
等实现在电视上直接观看云盘视频内容， 支持客户端 App 直接从夸克云盘获取文件播放而不经过运行本应用的服务器中转, 支持上传文件，但受限于 WebDAV 协议不支持文件秒传。


如果项目对你有帮助，欢迎 Star 或者赞助我，以支持本项目的继续开发

## 支付码

<p align="center">
  <img src="https://github.com/chenqimiao/chenqimiao/raw/main/pic/alipay.JPG" alt="alipay" width="400" height="400" style="margin-right: 40px;"/>
  <img src="https://github.com/chenqimiao/chenqimiao/raw/main/pic/wechat_pay.JPG" alt="wechat_pay" width="400" height="400"/>
</p>


> **Note**
>
> 本项目作者没有上传需求, 所以暂时还没有开发上传功能，后续考虑迭代

## 安装

可以从 [GitHub Releases](https://github.com/chenqimiao/quarkdrive-webdav/releases) 页面下载预先构建的二进制包


## Docker 运行

```bash
docker run -d --name=quarkdrive-webdav --restart=unless-stopped -p 8080:8080 \
  -e QUARK_COOKIE='you quark cookie' \
  -e WEBDAV_AUTH_USER=admin \
  -e WEBDAV_AUTH_PASSWORD=admin \
  ghcr.io/chenqimiao/quarkdrive-webdav:latest
```

其中，`QUARK_COOKIE` 环境变量为你的夸克云盘 `cookie`，`WEBDAV_AUTH_USER`
和 `WEBDAV_AUTH_PASSWORD` 为连接 WebDAV 服务的用户名和密码。



点击 Create (创建)后启动，用webdav客户端连接http://nas地址:8080 即可

## 🚨 免责声明

本项目仅供学习和研究目的，不得用于任何商业活动。用户在使用本项目时应遵守所在地区的法律法规，对于违法使用所导致的后果，本项目及作者不承担任何责任。
本项目可能存在未知的缺陷和风险（包括但不限于设备损坏和账号封禁等），使用者应自行承担使用本项目所产生的所有风险及责任。
作者不保证本项目的准确性、完整性、及时性、可靠性，也不承担任何因使用本项目而产生的任何损失或损害责任。
使用本项目即表示您已阅读并同意本免责声明的全部内容。



## 本项目参考了以下开源项目，特此鸣谢
- [aliyundrive-webdav](https://github.com/messense/aliyundrive-webdav)
