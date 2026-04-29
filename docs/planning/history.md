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
