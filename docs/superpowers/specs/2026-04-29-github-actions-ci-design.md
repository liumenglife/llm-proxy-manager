# GitHub Actions CI 设计说明

## 目标

为当前 Tauri v2 + React/TypeScript 项目提供统一的 GitHub Actions CI。CI 需要覆盖前端质量检查、Rust 质量检查、Tauri debug 打包和 Docker 镜像构建校验，确保合入前能发现类型、构建、格式、静态检查和容器构建问题。

## 触发条件

CI 在 `pull_request` 和 `push` 事件触发。这样既覆盖分支推送，也覆盖拉取请求的合入前验证。

## 作业拆分

CI 拆分为两个作业：

- `quality`：执行依赖安装、前端类型检查、前端 lint 分支、前端 build、Rust fmt、Rust clippy、Rust check 和 Rust test。
- `package`：依赖 `quality` 通过后执行，重新安装依赖并构建前端产物，然后运行 Tauri debug build 和 Docker build 校验。

这种拆分让基础质量门禁先失败，避免在明显质量问题存在时继续执行更重的打包与容器构建。

## 包管理器策略

仓库当前使用 `package-lock.json`，CI 固定执行 `npm ci`。由于当前依赖图存在第三方 peer dependency 版本不一致，仓库通过 `.npmrc` 固化 `legacy-peer-deps=true`，确保本地和 CI 的裸 `npm ci` 使用同一 npm 解析策略。工作流会先检测锁文件：

- 发现 `package-lock.json` 时输出 `Using npm ci because package-lock.json was found.` 并继续执行 `npm ci`。
- 发现 `pnpm-lock.yaml` 或 `yarn.lock` 时立即失败，并说明本仓库当前 CI 要求 npm。
- 没有 `package-lock.json` 时立即失败，并说明本仓库当前 CI 要求 npm 和 `package-lock.json`。

该策略满足“自动识别包管理器”的可观测性，同时保持本项目实际执行命令固定为 `npm ci`。

## 前端质量门禁

前端质量门禁包含三部分：

- 类型检查：执行 `npx tsc --noEmit`。
- lint：读取 `package.json` 的 `scripts.lint`。如果存在则执行 `npm run lint`；如果不存在则输出精确文本 `No lint script found, skipping frontend lint.`。
- 构建：执行 `npm run build`。

当前 `package.json` 没有 `lint` 脚本，因此 CI 不新增、不伪造 lint 脚本，只输出跳过文本。

## Rust 质量门禁

Rust 使用 `dtolnay/rust-toolchain@stable` 安装 stable toolchain。Rust 命令全部在 `src-tauri` 目录执行：

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo check --all-targets --all-features
cargo test --all-targets --all-features
```

其中 `clippy` 使用 `-D warnings`，保持 warning 作为失败项。

## Tauri 打包

`package` 作业执行 debug 打包：

```bash
npm run tauri build -- --debug
```

Linux runner 安装 Tauri v2 官方文档提到的 `libwebkit2gtk-4.1-dev`、`libayatana-appindicator3-dev`、`librsvg2-dev`、`patchelf`，并补充项目构建实际需要的 `build-essential`、`pkg-config`、`libssl-dev`、`libgtk-3-dev`、`libsoup-3.0-dev`、`libjavascriptcoregtk-4.1-dev`、`cmake`、`clang`、`libclang-dev`、`curl`、`wget`、`file`。

## Docker 校验

仓库包含 `docker/Dockerfile`、`docker/Dockerfile.backend` 和 `docker/Dockerfile.backend.localdist`。`package` 作业在 Docker build 前先执行 `npm ci` 和 `npm run build`，确保 `dist/` 已存在，满足 `Dockerfile.backend.localdist` 的输入要求。

Docker 校验命令为：

```bash
docker build -f docker/Dockerfile -t llm-proxy-manager:ci .
docker build -f docker/Dockerfile.backend --build-arg FRONTEND_IMAGE=llm-proxy-manager:ci -t llm-proxy-manager-backend:ci .
docker build -f docker/Dockerfile.backend.localdist -t llm-proxy-manager-backend-localdist:ci .
```

## 本地等价命令

本地可用以下命令验证 CI 的核心质量门禁：

```bash
npm ci
npx tsc --noEmit
node -e "const p=require('./package.json'); if (p.scripts && p.scripts.lint) process.exit(0); console.log('No lint script found, skipping frontend lint.')"
npm run build
cd src-tauri && cargo fmt --all -- --check
cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings
cd src-tauri && cargo check --all-targets --all-features
cd src-tauri && cargo test --all-targets --all-features
npm run tauri build -- --debug
docker build -f docker/Dockerfile -t llm-proxy-manager:ci .
docker build -f docker/Dockerfile.backend --build-arg FRONTEND_IMAGE=llm-proxy-manager:ci -t llm-proxy-manager-backend:ci .
docker build -f docker/Dockerfile.backend.localdist -t llm-proxy-manager-backend-localdist:ci .
```
