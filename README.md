# llm-proxy-Manager

`llm-proxy-Manager` 是一个基于 Tauri v2、React、TypeScript 和 Rust 的本地 AI 账号管理与协议代理工具。它将桌面管理界面、本地代理服务、账号管理、模型映射和 Docker Headless 部署整合到一个项目中。

## 核心能力

- 多 Provider 账号管理：支持 Gemini 与 Codex/OpenAI 账号分流。
- 协议代理：提供 OpenAI、Anthropic 与 Gemini 风格接口，便于不同客户端接入。
- Codex 接入：Codex OAuth 使用 OpenAI Auth0 + PKCE，不使用 GitHub OAuth，也不需要 `client_secret`。
- 模型映射：支持把客户端请求模型映射到实际上游模型。
- 本地桌面应用：Tauri v2 提供 macOS、Windows、Linux 桌面打包能力。
- Headless Docker：支持在服务器或 NAS 上运行 Web 管理界面与 API 代理。
- CI 质量门禁：GitHub Actions 覆盖前端构建、Rust 检查、Tauri debug build 与 Docker build。

## 快速开始

### 环境要求

- Node.js 与 npm
- Rust toolchain
- Tauri v2 所需系统依赖
- Docker，可选，仅 Docker 部署或镜像构建需要

### 本地开发

```bash
npm ci
npm run tauri dev
```

### 前端构建

```bash
npm run build
```

## 本地桌面打包

普通桌面打包不需要配置 `TAURI_SIGNING_PRIVATE_KEY`。

### macOS `.dmg`

在 macOS 机器的项目根目录执行：

```bash
npm ci
npm run tauri build -- --bundles dmg
```

产物路径：

```text
src-tauri/target/release/bundle/dmg/*.dmg
```

### Windows `.exe`

在 Windows 机器 PowerShell 的项目根目录执行：

```powershell
npm ci
npm run tauri build -- --bundles nsis
```

产物路径：

```text
src-tauri\target\release\bundle\nsis\*.exe
```

`.dmg` 推荐在 macOS 上编译，`.exe` 推荐在 Windows 上编译；macOS 交叉编译 Windows 不作为推荐路径。

## Docker 部署

```bash
docker run -d --name llm-proxy-manager \
  -p 8045:8045 \
  -e API_KEY=sk-llm-api-key \
  -e WEB_PASSWORD=your-login-password \
  -e ABV_MAX_BODY_SIZE=104857600 \
  -v ~/.llm_proxy_manager:/root/.llm_proxy_manager \
  llm-proxy-manager:latest
```

访问地址：

- Web 管理界面：`http://localhost:8045`
- API Base：`http://localhost:8045/v1`

更多 Docker 用法见 [`docker/README.md`](docker/README.md)。

## 本地构建 Docker 镜像

```bash
docker build -f docker/Dockerfile -t llm-proxy-manager:ci .
docker build -f docker/Dockerfile.backend --build-arg FRONTEND_IMAGE=llm-proxy-manager:ci -t llm-proxy-manager-backend:ci .
docker build -f docker/Dockerfile.backend.localdist -t llm-proxy-manager-backend-localdist:ci .
```

## 配置说明

| 配置 | 用途 |
| --- | --- |
| `API_KEY` / `ABV_API_KEY` | API 代理鉴权 Key，示例值：`sk-llm-api-key` |
| `WEB_PASSWORD` / `ABV_WEB_PASSWORD` | Web 管理后台登录密码 |
| `ABV_MAX_BODY_SIZE` | 最大请求体大小，默认示例为 `104857600` |
| `ABV_PUBLIC_URL` | 远程 OAuth 回调场景下的公网地址，可选 |

## 开发与验证

```bash
npm ci
npx tsc --noEmit
npm run build
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
npm run tauri build -- --debug
```

## 安全说明

- 不要把真实 OAuth token、API Key、账号数据或本地配置提交到仓库。
- Codex OAuth 使用 OpenAI Auth0 + PKCE，不需要 GitHub OAuth。
- 普通桌面打包不需要 `TAURI_SIGNING_PRIVATE_KEY`。
- Docker 部署建议同时配置 `API_KEY` 与 `WEB_PASSWORD`，区分 API 调用权限和 Web 管理权限。

## 文档入口

- [`docs/README.md`](docs/README.md)：开发文档索引
- [`docker/README.md`](docker/README.md)：Docker 部署说明
- [`docs/planning/current.md`](docs/planning/current.md)：当前规划状态
- [`docs/planning/decisions.md`](docs/planning/decisions.md)：关键决策记录
