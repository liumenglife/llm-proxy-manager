# llm-proxy-Manager Docker 部署指南

本目录包含 `llm-proxy-Manager` 的 Docker 部署文件，用于在服务器、NAS 或本地容器环境中运行 Web 管理界面与 API 代理服务。

## 快速启动

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

## Docker Compose

在项目根目录执行：

```bash
docker compose -f docker/docker-compose.yml up -d
```

当前 Compose 配置使用 `network_mode: host`，服务会直接监听宿主机的 `8045` 端口。

查看日志：

```bash
docker compose -f docker/docker-compose.yml logs -f --tail=200
```

## 本地构建镜像

在项目根目录执行完整镜像构建：

```bash
docker build -f docker/Dockerfile -t llm-proxy-manager:ci .
```

构建 backend 镜像：

```bash
docker build -f docker/Dockerfile.backend --build-arg FRONTEND_IMAGE=llm-proxy-manager:ci -t llm-proxy-manager-backend:ci .
```

复用本地 `dist/` 构建 backend localdist 镜像：

```bash
npm ci
npm run build
docker build -f docker/Dockerfile.backend.localdist -t llm-proxy-manager-backend-localdist:ci .
```

## localdist Compose

如果前端近期不变、后端需要频繁构建，可以先生成本地 `dist/`，再使用 localdist Compose：

```bash
npm ci
npm run build
docker compose -f docker/docker-compose.yml -f docker/docker-compose.localdist.yml up -d --build
```

## 环境变量

| 变量名 | 说明 |
| --- | --- |
| `API_KEY` / `ABV_API_KEY` | API 代理鉴权 Key，示例值：`sk-llm-api-key` |
| `WEB_PASSWORD` / `ABV_WEB_PASSWORD` | Web 管理后台登录密码 |
| `ABV_MAX_BODY_SIZE` | 最大请求体大小，示例值：`104857600` |
| `ABV_PUBLIC_URL` | 远程 OAuth 回调场景下的公网地址，可选 |
| `PORT` | 容器内服务监听端口，默认 `8045` |
| `LOG_LEVEL` | 日志等级，例如 `info`、`debug`、`warn`、`error` |

## 数据持久化

请挂载 `/root/.llm_proxy_manager`，避免账号、配置和本地状态在容器删除后丢失：

```bash
-v ~/.llm_proxy_manager:/root/.llm_proxy_manager
```

## 鉴权建议

- `API_KEY` 用于客户端调用 API 代理。
- `WEB_PASSWORD` 用于登录 Web 管理后台。
- 建议同时设置两者，避免 Web 管理权限和 API 调用权限共用同一个密钥。

## 常见排查

查看容器日志：

```bash
docker logs llm-proxy-manager
```

查看 Compose 日志：

```bash
docker compose -f docker/docker-compose.yml logs -f --tail=200
```

确认配置文件：

```bash
grep -E '"api_key"|"admin_password"' ~/.llm_proxy_manager/gui_config.json
```
