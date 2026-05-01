# llm-proxy-Manager

`llm-proxy-Manager` is a local AI account management and protocol proxy tool built with Tauri v2, React, TypeScript, and Rust. It combines a desktop management UI, a local proxy service, account management, model mapping, and Docker Headless deployment in one project.

## Core Capabilities

- Multi-provider account management for Gemini and Codex/OpenAI accounts.
- Protocol proxy endpoints compatible with OpenAI-style, Anthropic-style, and Gemini-style clients.
- Codex OAuth via OpenAI Auth0 + PKCE. It does not use GitHub OAuth and does not require `client_secret`.
- Model mapping from client-facing model names to upstream model names.
- Desktop packaging through Tauri v2 for macOS, Windows, and Linux.
- Headless Docker deployment for servers and NAS environments.
- CI quality gates covering frontend build, Rust checks, Tauri debug build, and Docker builds.

## Quick Start

### Requirements

- Node.js and npm
- Rust toolchain
- Tauri v2 system dependencies
- Docker, optional, only required for Docker deployment or image builds

### Local Development

```bash
npm ci
npm run tauri dev
```

### Frontend Build

```bash
npm run build
```

## Desktop Packaging

Regular desktop packaging does not require `TAURI_SIGNING_PRIVATE_KEY`.

### macOS `.dmg`

Run these commands from the project root on a macOS machine:

```bash
npm ci
npm run tauri build -- --bundles dmg
```

Output path:

```text
src-tauri/target/release/bundle/dmg/*.dmg
```

### Windows `.exe`

Run these commands from the project root in Windows PowerShell:

```powershell
npm ci
npm run tauri build -- --bundles nsis
```

Output path:

```text
src-tauri\target\release\bundle\nsis\*.exe
```

Build `.dmg` on macOS and `.exe` on Windows. Cross-compiling Windows installers from macOS is not the recommended path.

## Docker Deployment

```bash
docker run -d --name llm-proxy-manager \
  -p 8045:8045 \
  -e API_KEY=sk-llm-api-key \
  -e WEB_PASSWORD=your-login-password \
  -e ABV_MAX_BODY_SIZE=104857600 \
  -v ~/.llm_proxy_manager:/root/.llm_proxy_manager \
  llm-proxy-manager:latest
```

Endpoints:

- Web UI: `http://localhost:8045`
- API Base: `http://localhost:8045/v1`

See [`docker/README.md`](docker/README.md) for more Docker usage.

## Build Docker Images Locally

```bash
docker build -f docker/Dockerfile -t llm-proxy-manager:ci .
docker build -f docker/Dockerfile.backend --build-arg FRONTEND_IMAGE=llm-proxy-manager:ci -t llm-proxy-manager-backend:ci .
docker build -f docker/Dockerfile.backend.localdist -t llm-proxy-manager-backend-localdist:ci .
```

## Configuration

| Variable | Purpose |
| --- | --- |
| `API_KEY` / `ABV_API_KEY` | API proxy authentication key. Example: `sk-llm-api-key` |
| `WEB_PASSWORD` / `ABV_WEB_PASSWORD` | Web admin password |
| `ABV_MAX_BODY_SIZE` | Maximum request body size. Example: `104857600` |
| `ABV_PUBLIC_URL` | Optional public URL for remote OAuth callback scenarios |

## Development and Verification

```bash
npm ci
npx tsc --noEmit
npm run build
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
npm run tauri build -- --debug
```

## Security Notes

- Do not commit real OAuth tokens, API keys, account data, or local configuration files.
- Codex OAuth uses OpenAI Auth0 + PKCE and does not require GitHub OAuth.
- Regular desktop packaging does not require `TAURI_SIGNING_PRIVATE_KEY`.
- For Docker deployments, configure both `API_KEY` and `WEB_PASSWORD` to separate API access from Web admin access.

## Documentation

- [`docs/README.md`](docs/README.md): developer documentation index
- [`docker/README.md`](docker/README.md): Docker deployment guide
- [`docs/planning/current.md`](docs/planning/current.md): current planning state
- [`docs/planning/decisions.md`](docs/planning/decisions.md): decision log
