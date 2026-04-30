# 当前任务状态 (Current Planning)

## 1. 主目标
- [✓] 为当前 Tauri v2 + React/TypeScript 项目添加 GitHub Actions CI，并推送到当前 PR 分支。

## 2. 成功定义
- [✓] 新增 `.github/workflows/ci.yml`，在 `pull_request` 和 `push` 时触发。
- [✓] CI 拆为 `quality` 与 `package` 两个作业，`package` 依赖 `quality`。
- [✓] CI 使用 `npm ci`，不使用 pnpm/yarn；无 lint 脚本时输出 `No lint script found, skipping frontend lint.`。
- [✓] CI 包含 TypeScript、Vite build、Rust fmt/clippy/check/test、Tauri debug build 与三个 Dockerfile build 校验。
- [✓] 本地等价命令最终复审通过后提交并推送到当前 PR 分支。

## 3. 非目标
- 不新增或伪造前端 lint 脚本。
- 不通过无限增加 timeout 解决构建慢或本地验证失败。
- 不合并 PR。

## 4. 当前阶段
- [ ] 需求分析与架构设计 (Spec & Plan)
- [✓] CI 失败修复
- [•] PR CI 重新验证

## 5. 编码阶段任务清单
- [✓] 探索项目脚本、锁文件、Tauri/Rust/Docker 配置。
- [✓] 确认采用双作业 CI 方案：`quality` 与 `package`。
- [✓] 新增 CI 工作流与设计/计划文档。
- [✓] 诊断 `npm run build` 超时：确认脚本为 `tsc && vite build`，不包含 Tauri；Tauri debug build 卡在 `beforeBuildCommand` 后续阶段。
- [✓] 修复 Vite transform 慢点：移除 Lobe 图标依赖链、懒加载 `TokenStats`、调整 Vite 分包、补充 `tsconfig exclude`。
- [✓] 修复严格 `npm ci` 风险：删除未使用 Lobe 依赖，移除对 `legacy-peer-deps` 的依赖。
- [✓] 修复 `cargo clippy --all-targets --all-features -- -D warnings` 失败。
- [✓] 整理并修复 Rust 全量测试失败项；修复前禁止提交 CI。
- [✓] 重新验证 `cargo clippy --all-targets --all-features -- -D warnings` 与 `cargo test --all-targets --all-features`。
- [✓] 移除非必要 native updater 签名链路，解决 `TAURI_SIGNING_PRIVATE_KEY` 构建要求。
- [✓] 再验证 `npm run tauri build -- --debug`。
- [✓] 修复 PR #1 GitHub Actions `Quality / Rust clippy` 在 Rust stable 1.95.0 下失败的 10 个 lint 错误。
- [✓] 推送 clippy 修复后，PR #1 `Quality` job 已通过。
- [✓] 修复 PR #1 `Package / Docker build backend image` 的 `FRONTEND_IMAGE` ARG 作用域问题。
- [•] 提交并推送 Dockerfile 修复后，检查 PR #1 CI 状态。

## 6. 子 Agent 执行协议
- 遇到可以独立完成的编码任务，优先采用 Subagent-Driven Development。
- 主 Agent 负责拆解、派发、回收结果和更新全局真相，子 Agent 只处理局部任务。
- 子 Agent 返回后，主 Agent 再更新 `current.md` 和 `decisions.md`。

## 7. 历史任务归档
- `history.md` 保存所有已迁移批次的任务列表，保留每个 Task 的状态和结论。
- current.md 只保留当前批次，历史批次一律移走。
- 迁移时保持 Superpowers Todo 风格：`[✓]` 完成，`[ ]` 未开始，`[•]` 进行中。

## 8. Todo 状态说明
- `[✓]` 代表完成
- `[ ]` 代表未开始
- `[•]` 代表正在执行

## 9. 当前正在做
- PR #1 `Quality` job 已通过，`Package` job 新失败点已定位并修复，等待提交、推送并检查新一轮 PR CI。
- clippy 失败 run：`25153112782`，失败 job：`73728186352`。
- Docker 失败 run：`25156233365`，失败 job：`73739837190`。
- clippy 根因：GitHub Actions 使用 Rust stable `1.95.0`，本地先前验证环境为 Rust `1.94.0`，Clippy lint 集存在版本差异。
- Docker 根因：`docker/Dockerfile.backend` 的 `ARG FRONTEND_IMAGE` 声明位置不满足 `FROM ${FRONTEND_IMAGE}` 解析作用域，且 Dockerfile 内仍有旧 `/app/antigravity-tools` 路径。
- 本地复验：`cargo fmt --all -- --check`、`cargo +1.95.0 clippy --all-targets --all-features -- -D warnings`、`cargo +1.95.0 test --all-targets --all-features` 已通过。
- Docker 复验：`docker build --check -f docker/Dockerfile.backend ...`、`docker build --check -f docker/Dockerfile.backend.localdist ...`、backend/localdist 镜像构建与入口路径检查已通过。

## 10. 已完成里程碑
- [✓] 初始化 planning 文件体系（current.md / history.md / decisions.md）
- [✓] 批次 1：项目改名 Antigravity → llm-proxy-Manager
- [✓] 批次 2：Account 模型加 provider 字段
- [✓] 批次 3：Codex OAuth 授权（oauth_codex.rs + 回调服务器扩展 + 前端双按钮）
- [✓] 批次 4：Codex 代理 Handler（Provider trait + OpenAIProvider + Codex Handler + Token Manager 分流 + 模型映射）
- [✓] 批次 5：OpenCode 同步扩展（新增 llm-proxy-codex provider）
- [✓] 已创建 GitHub Actions CI 初版，包含 `quality` 与 `package` 作业。
- [✓] 已完成构建性能诊断与前端构建慢点修复：`npx vite build` 从超时改善为可完成。
- [✓] 已完成严格 npm 依赖修复：`npm ci --legacy-peer-deps=false` 已通过。
- [✓] 已完成 Rust clippy 修复：`cargo clippy --all-targets --all-features -- -D warnings` 已通过。
- [✓] 已完成 Rust 全量测试修复：`cargo test --all-targets --all-features` 为 `336 passed; 0 failed`。
- [✓] 已移除非必要 Tauri native updater 签名链路，`npm run tauri build -- --debug` 已通过。

## 11. 当前阻塞
- PR #1 需要推送 Dockerfile 修复提交后重新运行 GitHub Actions。

## 12. 活跃支线
- CI Docker 修复支线：修复 `Dockerfile.backend` 的 `FRONTEND_IMAGE` 作用域、Tauri 编译期 `dist` 可见性和旧 `/app/antigravity-tools` 路径残留。

## 13. 下一步唯一动作
- 提交并推送 Dockerfile 修复到 `feature/multi-provider`，然后检查 PR #1 CI 状态。

## 14. 恢复提示
- Session 恢复时，请检查此文件的状态，并沿着“当前阶段”与“下一步唯一动作”继续推进。
