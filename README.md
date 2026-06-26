# MicroBin 中文增强版

这是基于 [szabodanika/microbin](https://github.com/szabodanika/microbin) 的个人定制版本。原项目是一个轻量、自托管的 Pastebin / 文件分享 / URL 跳转服务。

本仓库由用户需求驱动，并由 AI 辅助完成代码修改、测试、打包与部署验证。主要目标是让 MicroBin 更适合中文环境下的临时分享、公开列表、密码访问和个人分享记录管理。

## 与原版的关系

- 上游项目：`szabodanika/microbin`
- 当前基础版本：基于上游 `master` 的 `67bbac5`
- 本仓库新增提交：`feat: add Chinese sharing workflow`
- 许可证：保留原项目 `BSD-3-Clause License`

本项目不是官方 MicroBin 版本，也不代表原作者对本定制版背书。

## 新增和调整功能

- 中文界面：导航、创建页、列表页、帮助页、查看/鉴权页面等主要文案已汉化。
- 自定义分享码：创建分享时可以填写更直观的分享码，例如 `work-note`、`1234`。
- 直接路径访问：分享码可以直接拼到域名后访问，例如 `https://example.com/1234`。
- 分享码搜索：公开列表页支持按分享码筛选。
- 公开但需要密码：内容仍显示在公开列表中，但查看正文、原始内容、文件或跳转时需要输入密码。
- 我的分享：浏览器本地记录自己创建过的分享，方便返回列表页找回链接。
- 失效自动隐藏：过期、阅后销毁或已删除的本地分享记录会自动从“我的分享”中隐藏，不再显示“已失效”。
- Hover 帮助提示：创建页问号支持鼠标悬停查看说明，同时保留点击进入完整帮助页。
- SQLite 兼容迁移：新增自定义分享码字段时会自动迁移旧数据库。
- Docker 运行时调整：镜像内包含必要运行时依赖，适配当前部署方式。

## 隐私和本地记录说明

“我的分享”只保存在当前浏览器的 `localStorage` 中，不会同步到服务器，也不会保存密码。

如果某条分享已经过期、达到阅后销毁次数、被删除，或服务返回 MicroBin 的 `404 Not Found` 页面，本地记录会自动移除。

## 配置说明

示例配置文件为 `.env.example`。部署前请复制为 `.env` 并按需修改，尤其是：

```bash
MICROBIN_ADMIN_PASSWORD=change-me
```

不要把真实 `.env`、数据库文件、上传文件或备份文件提交到公开仓库。

## 上传限制

当前默认上传限制：

- 加密或带密码上传：`256 MB`
- 不加密上传：`2048 MB`

可通过环境变量调整：

```bash
MICROBIN_MAX_FILE_SIZE_ENCRYPTED_MB=256
MICROBIN_MAX_FILE_SIZE_UNENCRYPTED_MB=2048
```

## 测试

仓库中包含一些轻量测试，用于覆盖本次定制功能：

```bash
node tests/my-shares.test.js
node tests/index-template.test.js
node tests/dockerfile-runtime.test.js
node tests/sqlite-migration.test.js
```

## 许可证

本项目继承原 MicroBin 的 [BSD 3-Clause License](LICENSE)。

原项目版权声明、许可证文本和免责声明已保留。使用、修改或分发本项目时，请继续遵守该许可证要求。
