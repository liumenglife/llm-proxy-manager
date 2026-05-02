# 关键决策记录 (Decisions Log)

本文件用于记录项目开发过程中涉及架构设计、命名规范、技术选型或重大变更的最终决策。

## 决策历史

### [2026-04-29] 1. 初始化项目规范
- **决策内容**：启用 `superpowers-planning-with-files` 工作流，以 `current.md` 维护主线状态。
- **决策原因**：为了在多 Agent 协作和跨 Session 开发中保持一致的全局真相，防止任务偏航。
- **影响范围**：整个项目生命周期内，主 Agent 需遵守该规范进行读写。

### [2026-04-29] 2. 项目更名为 llm-proxy-Manager
- **决策内容**：项目从 Antigravity Tools 更名为 llm-proxy-Manager，全项目文件同步。
- **决策原因**：原名称 Antigravity 不再反映项目的新定位和功能范围。
- **影响范围**：全项目—Rust 源码、前端源码、配置、文档、安装脚本、Docker、README。

### [2026-04-29] 3. Account 模型新增 provider 字段
- **决策内容**：Account 模型增加 `provider: String` 字段，取值为 `"gemini"` 或 `"codex"`，默认 `"gemini"`。
- **决策原因**：支持多 Provider（Gemini + OpenAI ChatGPT）的账号管理。
- **影响范围**：`src-tauri/src/models/account.rs`、`src/types/account.ts`、所有 `Account::new()` 调用点。

### [2026-04-29] 4. Codex OAuth 使用 OpenAI Auth0 + PKCE S256
- **决策内容**：Codex OAuth 使用 `https://auth.openai.com/authorize` 与 `https://auth0.openai.com/oauth/token`，通过公开客户端与 PKCE S256 完成授权，不使用 `client_secret`。
- **决策原因**：Codex 登录目标是 OpenAI ChatGPT/Codex 授权，不应引入 GitHub OAuth 或 GitHub 凭据。
- **影响范围**：`src-tauri/src/modules/oauth_codex.rs`、`src-tauri/src/modules/oauth_server.rs`、前端添加账号入口。

### [2026-04-29] 5. Provider 抽象 — AiProvider trait + 注册表
- **决策内容**：新增 `proxy/providers/` 模块，定义 `AiProvider` trait，Codex 实现 `OpenAIProvider`，后续 Gemini 也逐步迁移。
- **决策原因**：统一不同 AI 提供商的上游调用接口，使 token 刷新、请求发送、模型列表等逻辑可替换。
- **影响范围**：`src-tauri/src/proxy/providers/`、`token_manager.rs`、`model_mapping.rs`。

### [2026-04-29] 6. OpenCode 同步新增 llm-proxy-codex provider
- **决策内容**：在 OpenCode 同步中新增 `llm-proxy-codex` provider（`@ai-sdk/openai`），与 `antigravity-manager` provider（`@ai-sdk/anthropic`）并存。
- **决策原因**：Codex CLI 和 OpenCode 使用 OpenAI 协议，需独立的 OpenAI provider 配置。
- **影响范围**：`src-tauri/src/proxy/opencode_sync.rs`、`src/components/proxy/OpenCodeSyncModal.tsx`。

### [2026-04-29] 7. Rust warning 作为质量门禁失败项
- **决策内容**：当前批次提交前，`cargo check` 与指定 `cargo test` 输出必须达到零 warning。
- **决策原因**：用户要求之前质量审查发现的 warning 必须修复，避免带着隐患提交。
- **影响范围**：当前功能分支全部 Rust 代码与测试。

### [2026-04-29] 8. Web 管理 API 自启动切换接口必须实现
- **决策内容**：`POST /api/system/autostart/toggle` 不允许返回未实现状态，改为更新并持久化 `AppConfig.auto_launch`。
- **决策原因**：该接口已被前端请求层映射为 `toggle_auto_launch`，返回未实现会导致管理功能不可用。
- **影响范围**：`src-tauri/src/proxy/server.rs` 与对应测试。

### [2026-04-29] 9. GitHub Actions CI 采用双作业结构
- **决策内容**：CI 使用 `quality` 与 `package` 两个作业；`quality` 负责前端类型检查、lint 检测、前端构建、Rust fmt/clippy/check/test；`package` 依赖 `quality`，负责 Tauri debug build 与 Docker build 校验。
- **决策原因**：质量门禁与打包校验分离，便于定位 PR 失败阶段，同时满足用户要求。
- **影响范围**：`.github/workflows/ci.yml`。

### [2026-04-29] 10. CI 固定使用 npm ci
- **决策内容**：当前仓库 CI 固定使用 `npm ci`；如果发现 `pnpm-lock.yaml` 或 `yarn.lock`，CI 直接失败；无 lint 脚本时输出 `No lint script found, skipping frontend lint.`。
- **决策原因**：仓库当前只有 `package-lock.json`，用户明确要求不要使用 pnpm/yarn，不要伪造 lint 脚本。
- **影响范围**：`.github/workflows/ci.yml`、`package.json`、`package-lock.json`。

### [2026-04-29] 11. 移除未使用 Lobe 依赖以解决构建性能与 npm peer 冲突
- **决策内容**：移除未使用的 `@lobehub/icons`、`@lobehub/ui`、`@lobehub/fluent-emoji`，用本地轻量 SVG 替代模型图标，并保留 `TokenStats` 懒加载与 Vite 分包。
- **决策原因**：`@lobehub/icons` 图标组件链会拉入 Lobe UI、AntD 等大量依赖，导致 Vite transform 明显变慢；同时该依赖与当前 Ant Design 版本存在 peer dependency 冲突。
- **影响范围**：`package.json`、`package-lock.json`、`src/config/modelConfig.ts`、`src/App.tsx`、`vite.config.ts`。

### [2026-04-29] 12. 本地最终验证必须在磁盘空间恢复后继续
- **决策内容**：在当前机器磁盘空间不足时暂停最终验证，不通过继续增加 timeout 规避；重启并释放空间后再运行 `cargo test --all-targets --all-features` 与 `npm run tauri build -- --debug`。
- **决策原因**：最终复审已出现 `No space left on device`，继续验证会产生不可信结果并消耗更多磁盘。
- **影响范围**：当前 CI 支线的提交、推送与 PR CI 状态检查流程。

### [2026-04-29] 13. 移除非必要 native updater 签名链路
- **决策内容**：移除 Tauri native updater 自动下载/安装能力、updater pubkey、updater 权限、updater 前后端依赖和 `createUpdaterArtifacts`；保留普通后端版本检查与手动下载入口。
- **决策原因**：当前 updater 仍指向旧仓库且要求 `TAURI_SIGNING_PRIVATE_KEY`，该签名链路不是当前 CI 与核心代理业务的必要能力，会制造非必要密钥配置和构建失败。
- **影响范围**：`src-tauri/tauri.conf.json`、`src-tauri/Cargo.toml`、`src-tauri/Cargo.lock`、`src-tauri/src/lib.rs`、`src-tauri/capabilities/default.json`、`src/components/UpdateNotification.tsx`、`package.json`、`package-lock.json`、`.github/workflows/ci.yml`。

### [2026-05-01] 14. OAuth 活动 flow 必须按 provider 隔离
- **决策内容**：Gemini 与 Codex 不共享同一个活动 OAuth flow；复用旧 flow 前必须同时匹配 `provider` 与 OAuth client key。Codex OAuth 授权完成后必须保存为 `provider=codex` 的账号，并刷新账号池。
- **决策原因**：添加账号弹窗会同时预生成 Gemini 与 Codex 授权 URL；单一全局 flow 若只按 `code_rx` 判断可复用，会导致 Codex 回调被 Gemini flow 吞掉，前端停留在“正在等待授权...”。
- **影响范围**：`src-tauri/src/modules/oauth_server.rs`、`src-tauri/src/modules/account.rs`、`src-tauri/src/modules/account_service.rs`、`src-tauri/src/commands/mod.rs`。

### [2026-05-01] 15. OAuth URL 事件必须携带 provider
- **决策内容**：`oauth-url-generated` 事件 payload 必须包含 `url` 与 `provider`，前端按事件来源 provider 写入 Gemini 或 Codex URL 状态。
- **决策原因**：只按当前选中 provider 写入状态会产生异步竞态，导致较晚返回的 Gemini URL 被写入 Codex 状态或反向错写。
- **影响范围**：`src-tauri/src/modules/oauth_server.rs`、`src/components/accounts/AddAccountDialog.tsx`、`scripts/tests/oauth-review-regressions.test.mjs`。
