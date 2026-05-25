# Zeabur Web 快速验证

这是一个最小单容器 Web 快速验证适配路径。它不是桌面版发布路径，也不引入新的 kernel 概念或包边界。请把任何公网 URL 都视为不可信环境：使用一次性 volume、设置访问 token，不要在该快速验证 host 中保存真实 secret。

## 运行内容

仓库根目录的 `Dockerfile` 会构建：

- 使用 `npm ci` 和 `npm run build` 构建 `clients/web`
- 构建 Rust release `ygg` 二进制
- 在 runtime 镜像中用同一个端口提供 Web 静态文件和 host API
- 打包 Web 安装与 secret-store 快速验证所需的最小官方 lab manifest（`git-tools-lab`、`integrity-lab`、`install-lab`、`secret-store-lab`）

容器启动时执行：

```sh
ygg host serve --http 0.0.0.0:$PORT --data-dir /data --profile /data/profiles/default.yaml --static-dir /app/public --access-token "$YGG_HTTP_ACCESS_TOKEN"
```

同一个 HTTP 服务暴露：

- `GET /` 以及 `/app/public` 下的 Web 资源
- `POST /rpc`（配置 token 时需要 token）
- `GET /kernel/v1/event.subscribe/:session_id`（配置 token 时需要 token）
- `GET /surface-bundles/...`（公开只读浏览器 artifact）
- `GET /healthz`

设置 `YGG_HTTP_ACCESS_TOKEN` 后，`/rpc` 与 `/kernel/...` 路由需要 `Authorization: Bearer <token>`。浏览器 SSE 使用 `?access_token=<token>`，因为 EventSource 不能发送自定义 header。Web client 会从页面 URL 的 `?ygg_token=<token>` 读取一次，保存到 `localStorage`，从地址栏移除该参数，之后自动用于 RPC/SSE。

`/surface-bundles/...` 在该 quick-validation 部署中是公开前端 artifact，这样 sandboxed iframe 的 dynamic import、stylesheet、font、image 能稳定加载。不要把 secret 放进 bundle 或 asset。安全边界是 host RPC/kernel token 加 SurfaceHost bridge capability policy，而不是隐藏前端 JavaScript/CSS。

## Zeabur 配置

- 服务类型：从仓库根目录使用 Dockerfile
- 端口：Zeabur 未自动注入时可设置 `PORT`，默认 `8080`
- 健康检查：`GET /healthz`
- Volume：将持久化存储挂载到 `/data`

推荐环境变量：

| 变量 | 默认值 | 用途 |
| --- | --- | --- |
| `PORT` | `8080` | HTTP 监听端口；entrypoint 绑定 `0.0.0.0:$PORT`。 |
| `YGG_DATA_DIR` | `/data` | 持久化 Yggdrasil 数据目录。 |
| `YGG_PROFILE` | `default` | 在 `/data/profiles` 下创建/使用的安全 profile id；entrypoint 会拒绝不安全值。 |
| `YGG_STATIC_DIR` | `/app/public` | 要服务的已构建 Web 静态目录。 |
| `YGG_HTTP_ACCESS_TOKEN` | 未设置 | 公网 URL 强烈建议设置；保护 RPC/SSE/service 路由。 |
| `YGG_REQUIRE_ACCESS_TOKEN` | `0` | 设为 `1` 时，如果缺少 `YGG_HTTP_ACCESS_TOKEN`，容器会启动失败。 |

如果 `/data/profiles/$YGG_PROFILE.yaml` 不存在，entrypoint 会创建一个轻量 SQLite profile。也可以通过挂载或写入同一路径来替换为自定义 profile。

在 Zeabur/公网验证时，请设置随机 token，并第一次访问时在 URL 加上 `?ygg_token=<token>`。不要复用生产凭据，也不要在该验证实例中保存真实 provider secret。

## 本地 smoke test

```sh
docker build -t ygg-zeabur-quick .
docker run --rm -p 8080:8080 -v ygg-data:/data \
  -e YGG_HTTP_ACCESS_TOKEN=dev-token \
  -e YGG_REQUIRE_ACCESS_TOKEN=1 \
  ygg-zeabur-quick
curl http://127.0.0.1:8080/healthz
```

## 限制

- 仅用于快速验证；不替代正式桌面分发。
- access token 只是 quick-validation auth，不是生产级 session auth。
- Surface bundle/asset 是公开只读浏览器 artifact，绝不要嵌入 secret。
- 该适配器不引入新的官方包 namespace 或安装模型。
- 公网验证请使用一次性 `/data` volume，测试结束后删除。
- 不要在公网 quick-validation URL 中输入真实 API key 或敏感 secret。
- 内置 Web 应用是静态构建；本地热更新仍使用现有 Vite 工作流。
