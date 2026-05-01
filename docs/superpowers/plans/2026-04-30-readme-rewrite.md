# README 全量重写实施计划

## 目标

将根目录与文档目录中的 README 类文件重写为当前项目 `llm-proxy-Manager` 的真实文档入口，清理旧项目名称、旧仓库链接、赞助捐赠内容、个人推广和冗长历史更新记录。

本计划只覆盖文档实施，不修改生产代码，不新增脚本，不要求运行完整构建。

## 架构

采用“根 README 负责产品入口，子目录 README 负责专项说明”的文档架构。

- 根中英文 README 提供项目介绍、能力边界、安装运行、桌面打包、Docker、开发命令、配置与安全说明。
- `docker/README.md` 聚焦容器化部署、镜像构建、环境变量、持久化与排查。
- `docs/README.md` 聚焦仓库内文档索引，帮助开发者定位 planning、proxy、CI、specs、plans 与其他开发资料。
- 所有命令、路径、配置键名保持与当前仓库一致，中文正文默认使用简体中文。

## 文件职责

- `README.md`：中文主入口，面向使用者与开发者，提供最短可执行路径。
- `README_EN.md`：英文主入口，与中文 README 信息对齐，保持命令、路径和配置键一致。
- `docker/README.md`：Docker 专项说明，覆盖快速启动、Compose、本地构建、复用 `dist/`、环境变量、持久化、访问地址和常见排查。
- `docs/README.md`：文档索引，按 planning、proxy、CI、specs、plans 和其他开发文档组织入口。

## Task 1：中文 README

重写 `README.md`，内容包括项目简介、核心能力、支持的协议与客户端、快速开始、本地桌面打包、Docker 部署、开发与质量检查命令、关键配置、安全说明和相关文档入口。

要求使用当前事实：项目名 `llm-proxy-Manager`、npm 包名 `llm-proxy-manager`、版本 `4.1.32`、Tauri v2、React / TypeScript / Vite、Rust、`npm ci`、`npm run tauri dev`、`npm run build`、Tauri 打包命令、Docker 构建命令、`sk-llm-api-key` 示例。

## Task 2：英文 README

重写 `README_EN.md`，与 `README.md` 信息结构对齐，保留相同命令、路径和配置键名。

不得出现旧项目名称、旧仓库链接、赞助捐赠内容、作者个人推广和旧项目推荐。

## Task 3：Docker README

重写 `docker/README.md`，聚焦 Docker 使用场景。

必须覆盖快速启动、Docker Compose、本地构建镜像、本地 `dist/` 复用构建、环境变量、数据持久化、访问地址和常见排查。Docker 本地构建命令需要覆盖 `docker/Dockerfile`、`docker/Dockerfile.backend`、`docker/Dockerfile.backend.localdist`。

## Task 4：docs README

重写 `docs/README.md` 为简体中文文档索引。

必须覆盖 Planning 文档入口、Proxy 文档入口、CI / specs / plans 文档入口和其他开发文档入口。

## Task 5：全量验证

完成四个 README 文件后执行文本扫描和 diff 复核，确认旧内容清理、API Key 示例统一、改动范围符合预期。

验证命令：

```bash
rg -n "Antigravity-Manager|Antigravity Tools|lbjlaq/Antigravity-Manager|Buy Me|buymeacoffee|支付宝|微信支付|donate|Sponsor|赞助|Ctrler|Antigravity-Tools-LS|t\.me/AntigravityManager" README.md README_EN.md docker/README.md docs/README.md
rg -n "Antigravity" README.md README_EN.md docker/README.md docs/README.md
rg -n "sk-your-api-key|your-secret-key|API_KEY=.*your" README.md README_EN.md docker/README.md docs/README.md
git diff -- README.md README_EN.md docker/README.md docs/README.md docs/superpowers/specs/2026-04-30-readme-rewrite-design.md docs/superpowers/plans/2026-04-30-readme-rewrite.md
git status --short
```

验收要求：前三条 `rg` 命令不应返回命中；`git diff` 只包含预期文档改动；`git status --short` 不应出现生产代码改动。
