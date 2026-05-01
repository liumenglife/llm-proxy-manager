# README 全量重写设计

## 背景

当前根目录 `README.md` 与 `README_EN.md` 仍包含旧项目遗留内容，包括旧品牌、旧仓库链接、赞助与捐赠入口、个人推广、旧项目推荐和冗长历史更新记录。`docker/README.md` 与 `docs/README.md` 也存在命名、语言和索引风格不一致的问题。

本次目标是将所有 README 类文档收敛到当前项目 `llm-proxy-Manager` 的真实状态，形成可维护、可执行、无旧项目包袱的文档入口。

## 范围

需要重写或清理以下文件：

- `README.md`
- `README_EN.md`
- `docker/README.md`
- `docs/README.md`

## 非目标

- 不修改生产代码。
- 不新增安装脚本或发布脚本。
- 不恢复 Tauri updater signing，不引入 `TAURI_SIGNING_PRIVATE_KEY` 要求。
- 不保留旧项目历史更新日志。
- 不保留赞助、支付宝/微信二维码、Buy Me a Coffee、作者个人链接、旧项目推荐和营销内容。

## 内容策略

采用“精简产品文档型”方案：根 README 负责让用户快速理解、安装、运行、打包和开发；子目录 README 负责对应目录的专业说明。

### `README.md`

使用简体中文重写，建议结构：

- 项目简介
- 核心能力
- 支持的协议与客户端
- 快速开始
- 本地桌面打包：macOS `.dmg` 与 Windows `.exe`
- Docker 部署
- 开发与质量检查命令
- 关键配置
- 安全说明
- 相关文档入口

### `README_EN.md`

用英文重写，与中文 README 信息对齐。保留相同命令、路径和配置键名，避免出现旧项目名称、旧仓库链接或旧赞助内容。

### `docker/README.md`

使用简体中文重写，聚焦 Docker 使用：

- 快速启动
- Docker Compose
- 本地构建镜像
- 本地 `dist/` 复用构建
- 环境变量
- 数据持久化
- 访问地址
- 常见排查

### `docs/README.md`

使用简体中文重写为文档索引：

- Planning 文档入口
- Proxy 文档入口
- CI / specs / plans 文档入口
- 其他开发文档入口

## 必须保持一致的事实

- 项目名：`llm-proxy-Manager`
- npm 包名：`llm-proxy-manager`
- 当前版本：`4.1.32`
- 桌面框架：Tauri v2
- 前端：React / TypeScript / Vite
- 后端：Rust
- 本地开发：`npm ci`、`npm run tauri dev`
- 前端构建：`npm run build`
- Tauri 打包：`npm run tauri build -- --bundles dmg`、`npm run tauri build -- --bundles nsis`
- macOS `.dmg` 产物：`src-tauri/target/release/bundle/dmg/*.dmg`
- Windows `.exe` 产物：`src-tauri\target\release\bundle\nsis\*.exe`
- Docker 本地构建命令需要覆盖现有三个 Dockerfile：`docker/Dockerfile`、`docker/Dockerfile.backend`、`docker/Dockerfile.backend.localdist`
- 普通打包不需要 `TAURI_SIGNING_PRIVATE_KEY`
- Codex OAuth 使用 OpenAI Auth0 + PKCE，不使用 GitHub OAuth
- API Key 示例统一使用 `sk-llm-api-key`

## 清理规则

重写后，目标 README 文件中不得出现以下旧内容：

- `Antigravity-Manager`
- `Antigravity Tools`
- `lbjlaq/Antigravity-Manager`
- 赞助商表格
- 支付宝或微信二维码
- `Buy Me a Coffee`
- 旧项目推荐链接
- 旧更新日志大段内容

如果 `Antigravity` 出现在必须说明兼容旧客户端或历史迁移的上下文中，必须先确认具体理由；本次默认不保留。

## 验收标准

- 四个 README 文件均已按当前项目状态重写或清理。
- 根中英文 README 内容结构一致，命令与路径一致。
- 所有安装、开发、打包、Docker 命令可从当前仓库脚本或配置推导。
- 文档中不再包含赞助、捐赠、个人推广和旧项目营销内容。
- 文档中不再包含旧仓库安装链接。
- `README.md`、`README_EN.md`、`docker/README.md`、`docs/README.md` 通过文本扫描确认旧内容已清理。
- 本次只做文档改动，不要求运行完整构建。

## 验证方式

- 读取四个 README 文件确认结构与内容。
- 使用文本搜索确认旧名称、旧仓库、赞助、捐赠关键词已清理。
- 使用 `git diff -- README.md README_EN.md docker/README.md docs/README.md` 复核改动范围。
- 使用 `git status --short` 确认只有预期文档改动。
