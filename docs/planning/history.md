# 历史任务归档 (Task History)

## 1. 归档规则
- 这里保存所有已经从 current.md 迁移出来的任务批次。
- 每个批次都要保留原始状态，至少包含 `[✓]`、`[•]` 和 `[ ]` 标记。
- 不要把历史批次继续留在 current.md。

## 2. 历史批次
### [2026-04-29] Codex OAuth 与多协议代理接入
- [✓] 批次 1：项目改名 Antigravity → llm-proxy-Manager。
- [✓] 批次 2：Account 模型新增 provider 字段，用于区分 Gemini 与 Codex 账号。
- [✓] 批次 3：接入 Codex OAuth 授权，扩展回调服务器与前端添加账号入口。
- [✓] 批次 4：接入 Codex 代理能力，包含 Provider 抽象、OpenAIProvider、Codex Handler、Token Manager 分流与模型映射。
- [✓] 批次 5：OpenCode 同步新增 llm-proxy-codex provider。
- [✓] 质量修复 1：清零 Rust warning，修复 Codex OAuth 测试超时、OpenAIProvider 未实现占位与相关测试。
- [✓] 质量修复 2：实现 Web 管理 API 自启动切换接口，移除未实现返回。
- [✓] 最终质量审查：`cargo check` 零错误零 warning；指定 7 组 Rust 测试全部通过且零 warning；GitHub OAuth 残留、未实现占位、OpenAI Auth0 + PKCE 检查全部通过。

### [2026-04-29] GitHub Actions CI 接入
- [✓] 设计确认：采用 `quality` 与 `package` 两个作业。
- [✓] CI 文件创建：新增 `.github/workflows/ci.yml`，覆盖前端、Rust、Tauri debug build 与三个 Dockerfile build 校验。
- [✓] 文档创建：新增 CI 设计文档与实施计划文档。
- [✓] 构建性能诊断：`npm run build` 拆解为 `tsc && vite build`；Tauri build 的慢点先卡在前端 `beforeBuildCommand`。
- [✓] Vite 慢点修复：移除 Lobe 图标依赖链、懒加载 `TokenStats`、调整分包、补充 `tsconfig exclude`。
- [✓] npm 依赖修复：删除未使用 Lobe 依赖，`npm ci --legacy-peer-deps=false` 已通过。
- [✓] Rust clippy 修复：`cargo clippy --all-targets --all-features -- -D warnings` 已通过。
- [✓] Rust 全量测试修复：修复 mapper thinking 并发污染、安全数据库测试隔离、CIDR/性能、TokenManager、retry delay 等失败项。
- [✓] 非必要配置清理：移除 native updater 自动下载/安装和签名链路，不再要求 `TAURI_SIGNING_PRIVATE_KEY`。
- [✓] 最终 QA：`npm ci --legacy-peer-deps=false`、lint 检测、`npx tsc --noEmit`、`npx vite build`、`npm run build`、Rust fmt/clippy/check/test、`npm run tauri build -- --debug` 全部通过。
- [✓] PR CI 修复 1：修复 Rust stable `1.95.0` 下新增 clippy lint，PR #1 `Quality` job 通过。
- [✓] PR CI 修复 2：修复 `Dockerfile.backend` 的 `FRONTEND_IMAGE` ARG 作用域、Tauri 编译期 `dist` 可见性和旧 `/app/antigravity-tools` 路径残留。
- [✓] PR #1 最新 GitHub Actions checks 通过：`Quality` 与 `Package` 均为 pass。

### [2026-04-30] PR #1 合并与主干收尾
- [✓] 将默认主干迁移为 `main`，并确认 `origin/HEAD` 指向 `main`。
- [✓] 合并 PR #1 到 `main`，merge commit 为 `7e691f7a304263a04efe996434b6d9980e8ea285`。
- [✓] 清理本地与远程 `feature/multi-provider` 分支。
- [✓] 清理旧 `master` 分支与多余 worktree。
- [✓] 检查并删除不需要恢复的 stash，确认工作区干净且本地 `main` 与 `origin/main` 同步。

### [2026-04-30] README 全量重写与 PR #2 合并
- [✓] 写入 README 全量重写设计文档。
- [✓] 写入 README 全量重写实施计划。
- [✓] 重写 `README.md`。
- [✓] 重写 `README_EN.md`。
- [✓] 重写 `docker/README.md`。
- [✓] 重写 `docs/README.md`。
- [✓] 运行旧内容扫描：旧品牌、旧仓库、赞助、捐赠、二维码、请喝咖啡、旧项目推荐均无残留。
- [✓] 运行 API Key 示例扫描，确认未使用旧占位符。
- [✓] 运行 `npm run build`，构建通过。
- [✓] 提交文档重写改动。
- [✓] 推送 `feature/readme-rewrite` 到 `origin`。
- [✓] 创建 base 为 `main` 的 PR：`https://github.com/liumenglife/llm-proxy-manager/pull/2`。
- [✓] PR #2 GitHub Actions `Quality` 与 `Package` checks 全部通过。
- [✓] 合并 PR #2 到 `main`，merge commit 为 `5ec2b64166eee3e1dff267dbc44a40ab3ccf193c`。
- [✓] 同步本地 `main` 到 `origin/main`。
- [✓] 清理本地与远程 `feature/readme-rewrite` 分支及对应 worktree。

### [2026-05-01] 禁用旧 updater 自动检查与旧发布入口清理
- [✓] 从最新 `main` 创建 `feature/fix-updater-config` 隔离 worktree。
- [✓] 全仓排查 updater、update、latest、release、endpoint、antigravity、github、owner、repo、download、signature、pubkey 等关键词。
- [✓] 确认 Tauri native updater 未启用：无 `tauri-plugin-updater`、无 `@tauri-apps/plugin-updater`、无 updater `pubkey` / `signature` 配置。
- [✓] 定位根因：自研 GitHub release 检查链路仍在启动后自动触发，且旧发布 endpoint 残留。
- [✓] 移除前端启动时自动更新检查与 `UpdateNotification` 渲染链路。
- [✓] 将后端更新设置默认值与异常兜底改为 `auto_check=false`。
- [✓] 将 `check_for_updates()` 改为当前阶段直接返回无更新且不访问网络。
- [✓] 移除自动检查 command、前端 request 映射和管理 API 自动检查入口。
- [✓] 移除旧 updater 服务、旧仓库 release endpoint、旧下载地址和活动发布入口旧应用名。
- [✓] 更新安装脚本、Cask、Arch 发布模板、官网静态页和运行时标题为当前仓库与当前项目名。
- [✓] 删除旧仓库 PR 关闭历史说明脚本文档。
- [✓] 子 Agent 按 TDD 添加并通过 updater、constants、server fallback 回归测试。
- [✓] 独立 QA 通过：专项测试、`cargo test --lib`、`cargo check --all-targets --all-features`、`npm run build`、旧词静态扫描全部通过。
- [✓] 本地 Tauri debug 启动验证已尝试，受首次 dev 编译耗时影响未在工具超时内进入运行阶段；启动链路由源码扫描和回归测试覆盖。

### [2026-05-01] PR #4 合并与 CI 触发修复收口
- [✓] 用户确认检查 PR #4 diff 无异常后允许合并。
- [✓] 确认 PR #4 `Quality` 与 `Package` 检查通过。
- [✓] 合并 PR #4 到 `main`，merge commit 为 `382b3d61d9d60b00365ab7556e49cdc43a65f903`。
- [✓] 同步本地 `main` 到 `origin/main`。
- [✓] 清理本地 `fix/pr3-ci-workflow` 分支、远程分支和 `.worktrees/fix-pr3-ci`。
- [✓] 当前保留主工作区 `main` 与 `.worktrees/feature-fix-updater-config`。
