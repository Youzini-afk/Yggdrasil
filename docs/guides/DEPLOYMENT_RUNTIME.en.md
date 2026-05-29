# Deployment Runtime

> [English](./DEPLOYMENT_RUNTIME.en.md) · [中文](./DEPLOYMENT_RUNTIME.md)

Yggdrasil can now host self-hosted AI / agent projects. Deployment is not a Docker concept in the kernel. It is a small set of generic runtime primitives that ordinary packages compose into Docker, native process, or future remote targets.

## Boundary

The kernel exposes four generic primitive families:

| Primitive | Protocol family | Role |
|---|---|---|
| target | `kernel.v1.target.*` | Describe an execution target. The built-in target is `local`. |
| exec | `kernel.v1.exec.*` | Start, stop, and inspect controlled local execution. Deny-all by default. |
| port | `kernel.v1.port.*` | Lease a loopback port from the host. |
| proxy | `kernel.v1.proxy.*` | Bind a public HTTP / WebSocket route to a port lease. |

Docker, git, installation, secret storage, workspaces, and adapters are not kernel concepts. Ordinary capability packages implement them.

## Current implementation

- `LocalExecExecutor` trait: defaults to `DenyAllLocalExecExecutor`; profiles may opt into `LiveLocalExecExecutor`.
- `LiveLocalExecExecutor`: accepts argv arrays only, never shell strings. cwd, env, logs, timeout, and kill behavior are host-controlled.
- `ygg-service` reverse proxy: `/p/<route_id>/...` routes through `kernel.v1.proxy.*` records and can only point at active loopback port leases. Redirects are disabled, dangerous response headers are stripped, response bodies are bounded, and HTTP + WebSocket are supported.
- `official/docker-runtime-lab`: an ordinary official capability package using `bollard` to manage Docker containers. It fails closed when Docker is unavailable; real Docker smoke requires opt-in.
- Web project console: shows target / exec / port / proxy diagnostics. If a project declares a Docker deployment descriptor, the user can explicitly click Deploy / Stop.

## Docker deployment descriptor

Native projects can add minimal deployment metadata under `project.metadata.deployment.docker` in `project.yaml`:

```yaml
project:
  metadata:
    deployment:
      docker:
        image: ghcr.io/example/app:latest
        container_port: 3000
        port_name: web        # optional, default: web
        route_id: my-app-web  # optional, default: <project_id>-web
        health_path: /healthz # optional, display only for now
        pull_if_missing: false
```

The current web broker accepts only these fields. `env`, `volumes`, `mounts`, `binds`, and `secrets` are rejected so the first deployment path cannot silently expand authority.

## Explicit Deploy flow

The Deploy button in the project console never runs automatically. After user confirmation, the web shell acts as a host broker:

1. `kernel.v1.port.lease`: lease a loopback port.
2. `kernel.v1.capability.invoke` → `official/docker-runtime-lab/start_container`: start the Docker container with `approved: true`, `host_port`, and `port_lease_id`.
3. `kernel.v1.proxy.register`: bind a route to that port lease.

If any step fails, the broker best-effort rolls back: unregister proxy, stop the known container from the current page, and release the port lease.

Stop deployment only stops the container id known to the current page. It does not stop unknown containers based on a matching `port_name`.

## `project.start` does not deploy

`kernel.v1.project.start` remains a project state machine: open or reuse a project session, mark Running, and return `session_id`. It does not start a process, allocate a port, or register a proxy.

Deployment is a separate, explicit host-broker action. This keeps “open project UI” and “run an external service” visibly separate.

## Red lines

- Deny-all by default. No profile opt-in means no real local process start.
- Ports bind to loopback only.
- Proxy upstreams must reference active port leases, and `port_name` must match.
- The reverse proxy does not follow upstream redirects and strips `Set-Cookie`, `Location`, CORS, and other dangerous response headers.
- Container deployment does not accept arbitrary env / volume / secret fields.
- Docker is implemented by an ordinary capability package. No official fast path.

## Next

- Native execution remains trusted/dev-oriented. It is not a full OS sandbox.
- Docker descriptors still lack pull progress, health polling, log archival, and volume policy.
- External project wizards can generate deployment descriptors later, but deployment must remain an explicit user action.
