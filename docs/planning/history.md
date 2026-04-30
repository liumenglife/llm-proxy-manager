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
- [•] 待完成：提交、推送到当前 PR 分支，并检查 PR CI 状态。
