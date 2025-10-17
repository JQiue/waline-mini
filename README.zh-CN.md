# waline-mini

![GitHub Release](https://img.shields.io/github/v/release/JQiue/waline-mini)
![GitHub Issues or Pull Requests](https://img.shields.io/github/issues/JQiue/waline-mini)
![GitHub commit activity](https://img.shields.io/github/commit-activity/t/JQiue/waline-mini)
![GitHub Downloads (all assets, all releases)](https://img.shields.io/github/downloads/JQiue/waline-mini/total)
![GitHub License](https://img.shields.io/github/license/JQiue/waline-mini)
![Code Lines](https://img.shields.io/endpoint?url=https://ghloc.vercel.app/api/JQiue/waline-mini/badge?filter=.rs$)
![Docker Image Size](https://img.shields.io/docker/image-size/jqiue/waline-mini)
![Docker Image Version](https://img.shields.io/docker/v/jqiue/waline-mini?label=docker)

[English](./README.md) | 简体中文

> 一个使用 Rust 实现的高性能 Waline 评论系统

## 介绍

Waline-mini 是原 Waline 评论系统的 Rust 实现，使用的内存比 Node.js 少 95%，是资源受限服务器的替代方案

在我的 Ubuntu 服务器上，waline-mini 仅需要 `5612kb=5.48mb`的内存占用

![mem](./assets/image.png)

+ **极低的内存使用率**: 只有 Node.js 版本内存占用的 1/25
+ **零依赖部署**: 无需安装 Node.js 环境，只需要一个可执行文件即可运行
+ **轻松替换**: 实现了原 Waline 大部分必要的 API
+ **同步更新**: 与原 Waline 的更新保持同步

## Waline 功能实现

| 功能                     | 可用性   | 状态   |
| ------------------------ | -------- | ------ |
| 页面浏览人数计数器       | 完全可用 | 稳定   |
| 文章反应                 | 完全可用 | 稳定   |
| 评论格式支持             | 完全可用 | 稳定   |
| 用户标签                 | 完全可用 | 稳定   |
| 国际化                   | 完全可用 | 稳定   |
| 邮件通知                 | 几乎可用 | 进行中 |
| 安全性：跨站脚本攻击     | 完全可用 | 稳定   |
| 安全性：频率限制         | 完全可用 | 稳定   |
| 安全性：防止灌水         | 完全可用 | 稳定   |
| 安全性：评论审核         | 完全可用 | 稳定   |
| 安全性：反垃圾评论       | 完全可用 | 稳定   |
| 安全性：违禁词           | 完全可用 | 稳定   |
| 安全性：安全域名         | 完全可用 | 稳定   |
| 安全性：不允许的 IP 列表 | 完全可用 | 稳定   |
| OAuth                    | 完全可用 | 稳定   |
| 数据迁移                 | 完全可用 | 稳定   |
| 双因素认证               | 完全可用 | 稳定   |

## 使用方法

### 从可执行文件中运行

从 [GitHub Releases](https://github.com/JQiue/waline-mini/releases) 下载对应平台的可执行文件，以 Linux + SQLite 使用示例：

```bash
# 设置必要的环境变量
export DATABASE_URL=sqlite:///path/to/waline.sqlite?mode=rwc
export JWT_TOKEN=your_secret_key
export SITE_NAME=your_site_name
export SITE_URL=your_site_url

# 启动
./waline-mini
```

### Docker

```yml
services:
  waline:
    image: jqiue/waline-mini:latest
    container_name: waline-mini
    ports:
      - "8360:8360"
    volumes:
      - waline-db:/app/db
    environment:
      - DATABASE_URL=${DATABASE_URL:-sqlite:////app/db/waline.sqlite?mode=rwc}
      - JWT_TOKEN=${JWT_TOKEN}
      - SITE_NAME=${SITE_NAME}
      - SITE_URL=${SITE_URL}
    restart: unless-stopped

volumes:
  waline-db:
    driver: local
```

如果想使用别的数据库只需要添加`-e DATABASE_URL`环境进行覆盖即可

### Shuttle

waline-mini 支持部署在 Shuttle 上，首先使用以下命令克隆`shuttle`分支到本地

```sh
git clone -b shuttle https://github.com/JQiue/waline-mini.git
```

然后，在项目根目录创建一个`.shuttle.env`用于配置 waline-mini 的环境变量

最后按照 [Shuttle](https://console.shuttle.dev/login) 的步骤进行部署

### LeanCloud

> LeanCloud 国内版不提供自定义域名，国际版虽提供域名但国内无法访问，请自行权衡

使用 LeanCloud 直接拉取仓库进行部署，分支需要填写`leancloud`

使用 SQLite 作为数据存储，则环境变量`DATABASE_URL`应配置为`sqlite://./waline.sqlite?mode=rwc`。在 LeanCloud 部署时，每次部署都会得到一个全新的 SQLite 文件，因此，在重新部署前务必导出数据，并在重新部署后将数据导入，升级 waline-mini 需要重新进行部署时，此步骤非常重要

## 配置

用环境变量配置 waline-mini:

| 环境变量               | 描述                                                                                                                        | 是否需要 | 默认值         |
| ---------------------- | --------------------------------------------------------------------------------------------------------------------------- | -------- | -------------- |
| DATABASE_URL           | SQLite and MySQL/MariaDB 是支持的，随时可以添加编译特性对 PostgreSQL 进行支持。`protocol://username:password@host/database` | ✅        | -              |
| JWT_TOKEN              | 使用一个随机字符串来生成 JWT 签名密钥 key                                                                                   | ✅        | -              |
| SITE_NAME              | 网站名称                                                                                                                    | ✅        | -              |
| SITE_URL               | 网站地址                                                                                                                    | ✅        | -              |
| SERVER_URL             | 自定义服务器地址                                                                                                            |          | auto           |
| HOST                   | 监听地址                                                                                                                    |          | `127.0.0.1`    |
| PORT                   | 监听端口                                                                                                                    |          | `8360`         |
| WORKERS                | 工作线程数                                                                                                                  |          | `1`            |
| LEVELS                 | 根据评论的数量给每个用户一个评级标签                                                                                        |          | -              |
| SMTP_SERVICE           | SMTP 邮件服务提供商：`QQ`，`GMail`，`126`，`163`                                                                            |          | -              |
| SMTP_HOST              | SMTP 服务器地址                                                                                                             |          | -              |
| SMTP_PORT              | SMTP 服务器端口                                                                                                             |          | -              |
| SMTP_USER              | SMTP 用户名                                                                                                                 |          | -              |
| SMTP_PASS              | SMTP 密码                                                                                                                   |          | -              |
| AUTHOR_EMAIL           | 博主的邮箱，用来判断发表的评论是否是博主发表的。如果是由博主发布的，则不会有提醒通知                                        |          | -              |
| IPQPS                  | 基于 ip 的评论发布频率以秒为单位限制。设置为`0`表示没有限制                                                                 |          | `60`           |
| COMMENT_AUDIT          | 评论审查开关。启用后，每个评论都需要由管理员批准，因此建议在占位符中提示                                                    |          | `false`        |
| AKISMET_KEY            | Akismet 反垃圾评论服务 Key (默认开启，不用请设置为`false`)                                                                  |          | `86fe49f5ea50` |
| LOGIN                  | 当设置为`LOGIN=force`时会要求登录才能评论                                                                                   |          | `false`        |
| FORBIDDEN_WORDS        | 违禁词配置，包含违禁词的内容会直接标记为垃圾评论                                                                            |          |                |
| DISALLOW_IP_LIST       | IP 黑名单配置，名单中的 IP 访问会直接返回 403 错误，比如：`8.8.8.8,3.3.3.3`                                                 |          |                |
| SECURE_DOMIANS         | 安全域名配置。配置后非该域名来源的请求会返回 403 状态码。支持字符串、正则、数组类型，不配置表示允许所有域名来源             |          |                |
| DISABLE_AUTHORE_NOTIFY | 是否禁止新评论通知                                                                                                          |          | `false`        |
| DISABLE_REGION         | 是否隐藏评论者的归属地                                                                                                      |          | `false`        |
| DISABLE_USERAGENT      | 是否隐藏评论者的 UA                                                                                                         |          | `false`        |
| IP2REGION_DB           | 自定义 IP 查询库路径，waline-mini 并不包含 xdb 文件，需要手动提供                                                           |          |                |

## 常见问题

### 如何迁移现有 Waline 数据？

1. 从原 waline 后台管理页面导出 JSON
2. 在 waline-mini 后台管理页面导入 JSON

### 支持哪些数据库？

目前支持 SQLite 和 MySQL/MariaDB，可以通过添加编译特性支持 PostgreSQL

## 参考

+ [waline-api](https://waline.js.org/next/api/)
