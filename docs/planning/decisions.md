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
