# GitHub Actions CI 实施计划

> **给代理执行者：** 必须使用 `superpowers:subagent-driven-development`（推荐）或 `superpowers:executing-plans` 按任务逐步执行本计划。步骤使用复选框（`- [ ]`）语法跟踪。

**目标：** 为当前 Tauri v2 + React/TypeScript 项目添加覆盖质量门禁、Tauri debug 打包和 Docker build 校验的 GitHub Actions CI。

**架构：** 使用 `.github/workflows/ci.yml` 定义两个作业：`quality` 负责快速质量门禁，`package` 在 `quality` 通过后执行较重的打包与容器构建。两个作业都固定使用 npm、Node LTS、Rust stable，并安装 Tauri v2 Linux 构建依赖。

**技术栈：** GitHub Actions、Node.js、npm、TypeScript、Vite、Tauri v2、Rust stable、Cargo、Docker。

---

## 文件结构

- 创建：`.github/workflows/ci.yml`，定义 `pull_request` 和 `push` 触发的双作业 CI。
- 创建：`.npmrc`，固化当前锁文件需要的 npm peer dependency 解析策略，保证裸 `npm ci` 可重复执行。
- 创建：`docs/superpowers/specs/2026-04-29-github-actions-ci-design.md`，记录 CI 设计、门禁策略和本地等价命令。
- 创建：`docs/superpowers/plans/2026-04-29-github-actions-ci.md`，记录本实施计划和验证步骤。

### 任务 1：创建 GitHub Actions 工作流

**文件：**
- 创建：`.github/workflows/ci.yml`
- 创建：`.npmrc`

- [ ] **步骤 1：创建工作流目录**

运行：`mkdir -p .github/workflows`
预期：`.github/workflows` 目录存在。

- [ ] **步骤 2：写入 CI 工作流**

将 `.github/workflows/ci.yml` 写为包含以下关键结构的 YAML：

```yaml
name: CI

on:
  pull_request:
  push:

jobs:
  quality:
    runs-on: ubuntu-22.04
  package:
    runs-on: ubuntu-22.04
    needs: quality
```

预期：工作流包含 `quality` 与 `package` 两个作业，且 `package` 使用 `needs: quality`。

- [ ] **步骤 3：配置工具链与依赖安装**

在两个作业中都加入 `actions/checkout@v4`、`actions/setup-node@v4`、`dtolnay/rust-toolchain@stable`，并安装 Tauri Linux 依赖。

预期：Linux 依赖至少包含 `libwebkit2gtk-4.1-dev`、`libayatana-appindicator3-dev`、`librsvg2-dev`、`patchelf`、`build-essential`、`pkg-config`、`libssl-dev`、`libgtk-3-dev`、`libsoup-3.0-dev`、`libjavascriptcoregtk-4.1-dev`、`cmake`、`clang`、`libclang-dev`、`curl`、`wget`、`file`。

- [ ] **步骤 4：配置 npm 锁文件策略**

在两个作业中都加入锁文件检测步骤：如果发现 `pnpm-lock.yaml` 或 `yarn.lock` 则失败；如果发现 `package-lock.json` 则执行 `npm ci`。

预期：CI 实际安装命令固定为 `npm ci`。

- [ ] **步骤 5：固化 npm peer dependency 策略**

写入 `.npmrc`：

```ini
legacy-peer-deps=true
```

预期：本地和 CI 的裸 `npm ci` 都使用同一 npm 解析策略。

### 任务 2：配置质量门禁与打包校验

**文件：**
- 修改：`.github/workflows/ci.yml`

- [ ] **步骤 1：配置前端质量门禁**

在 `quality` 作业中依次执行：

```bash
npm ci
npx tsc --noEmit
npm run build
```

预期：类型检查和前端构建失败时阻断 CI。

- [ ] **步骤 2：配置 lint 条件分支**

在 `quality` 作业中检测 `package.json` 是否存在 `scripts.lint`。存在时执行 `npm run lint`，不存在时输出：

```text
No lint script found, skipping frontend lint.
```

预期：当前仓库没有 lint 脚本，CI 日志输出上述精确文本。

- [ ] **步骤 3：配置 Rust 质量门禁**

在 `quality` 作业的 `src-tauri` 工作目录中执行：

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo check --all-targets --all-features
cargo test --all-targets --all-features
```

预期：Rust 格式、静态检查、编译检查和测试失败时阻断 CI。

- [ ] **步骤 4：配置 Tauri debug build**

在 `package` 作业中执行：

```bash
npm ci
npm run build
npm run tauri build -- --debug
```

预期：`package` 作业在打包前生成 `dist/`。

- [ ] **步骤 5：配置 Docker build 校验**

在 `package` 作业中执行：

```bash
docker build -f docker/Dockerfile -t llm-proxy-manager:ci .
docker build -f docker/Dockerfile.backend --build-arg FRONTEND_IMAGE=llm-proxy-manager:ci -t llm-proxy-manager-backend:ci .
docker build -f docker/Dockerfile.backend.localdist -t llm-proxy-manager-backend-localdist:ci .
```

预期：三个 Dockerfile 都能被构建校验。

### 任务 3：创建设计文档

**文件：**
- 创建：`docs/superpowers/specs/2026-04-29-github-actions-ci-design.md`

- [ ] **步骤 1：写入设计说明**

文档必须包含以下章节：目标、触发条件、作业拆分、包管理器策略、前端质量门禁、Rust 质量门禁、Tauri 打包、Docker 校验、本地等价命令。

预期：文档使用简体中文，并能解释每个 CI 设计决策。

### 任务 4：执行本地验证

**文件：**
- 验证：`.github/workflows/ci.yml`
- 验证：`docs/superpowers/specs/2026-04-29-github-actions-ci-design.md`
- 验证：`docs/superpowers/plans/2026-04-29-github-actions-ci.md`

- [ ] **步骤 1：安装前端依赖**

运行：`npm ci`
预期：依赖安装成功。

- [ ] **步骤 2：执行类型检查**

运行：`npx tsc --noEmit`
预期：命令成功退出。

- [ ] **步骤 3：执行 lint 等价检测**

运行：`node -e "const p=require('./package.json'); if (p.scripts && p.scripts.lint) process.exit(0); console.log('No lint script found, skipping frontend lint.')"`
预期：输出 `No lint script found, skipping frontend lint.`。

- [ ] **步骤 4：执行前端构建**

运行：`npm run build`
预期：命令成功退出并生成 `dist/`。

- [ ] **步骤 5：执行 Rust 格式检查**

运行：`cargo fmt --all -- --check`
工作目录：`src-tauri`
预期：命令成功退出。

- [ ] **步骤 6：执行 Rust 编译检查**

运行：`cargo check --all-targets --all-features`
工作目录：`src-tauri`
预期：命令成功退出。

- [ ] **步骤 7：检查 YAML 结构**

运行：`ruby -e "require 'yaml'; YAML.load_file('.github/workflows/ci.yml'); puts 'YAML syntax ok'"`
预期：输出 `YAML syntax ok`。
