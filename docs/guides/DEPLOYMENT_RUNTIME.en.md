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
- Web project console: shows target / exec / port / proxy diagnostics. If a project declares deployment metadata, the user can explicitly click Deploy / Stop, or start a Build & Deploy job.
- Persistence and replay: exec / port / proxy registry mutations are written to the event log and replayed to rebuild registries on host restart.
- Restart reconciliation: after a restart, replayed records are first downgraded (exec → unknown, port → reserved, proxy → stale with `ready=false`), then reconciled against the real world.
- Readiness gating: proxy routes register with `ready=false`; the reverse proxy returns 503 for routes that are not yet ready, and only forwards once ready.
- Health supervision: a host background loop periodically probes each active route's upstream, flips `ready=false` on sustained failure and `ready=true` on recovery, and writes an audit event on each transition.
- Build & Deploy broker: `POST /host/v1/build-deploy` creates a short-lived job. The host clones source, builds via Dockerfile / nixpacks, starts the container, registers proxy, and runs readiness probing. The browser observes job status / SSE and may cancel the job.

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
        health_path: /healthz # optional, used for the readiness probe
        pull_if_missing: false
```

The current web broker accepts only these fields. `env`, `volumes`, `mounts`, `binds`, and `secrets` are rejected so the first deployment path cannot silently expand authority.

## Build & Deploy descriptor

If a project has no prebuilt image, it can declare source build metadata under `project.metadata.deployment.build_deploy`:

```yaml
project:
  metadata:
    deployment:
      build_deploy:
        source_url: https://github.com/example/app.git
        ref_name: HEAD
        strategy: dockerfile # dockerfile | nixpacks
        dockerfile_path: Dockerfile # optional
        container_port: 3000
        port_name: web
        route_id: my-app-web
        health_path: /healthz
        runtime_env:
          - name: NODE_ENV
            value: production
          - name: OPENAI_API_KEY
            secret_ref: project:OPENAI_API_KEY
        runtime_mounts:
          - source_host_path: /srv/ygg-data/my-app
            container_path: /app/data
            mode: ro
            approved: true
            high_risk_approved: false
            reason: persistent app data
```

The `dockerfile` strategy uses a Dockerfile from the repository. The `nixpacks` strategy first runs local `nixpacks` to generate Dockerfile / context, then `docker-runtime-lab` builds the image through Docker. If `nixpacks` is unavailable, the build fails closed.

Runtime secrets must be `store:` / `project:` / `env:` `secret_ref`s. Raw secret values are injected by a host-private Docker runner and never cross the `docker-runtime-lab` package boundary or enter events, logs, or job state. Build-time secrets are not supported yet and fail closed.

Volumes may point to arbitrary host paths, but every mount needs explicit approval. Read-only is the default recommendation; read-write mounts require an extra confirmation. The host denies Docker sockets, system directories, secret directories, Yggdrasil secret storage, broad home directories, and ancestor paths that would implicitly include them.

## Explicit Deploy flow

The Deploy button in the project console never runs automatically. After user confirmation, the request is sent to the host-plane `POST /host/v1/deploy`, where the host broker drives the whole chain server-side (the browser is a thin client and no longer orchestrates):

1. The host re-validates the request (client fields are not trusted).
2. `kernel.v1.port.lease`: lease a loopback port.
3. `kernel.v1.capability.invoke` → `official/docker-runtime-lab/start_container`: start the Docker container with `approved: true`, `host_port`, and `port_lease_id`.
4. `kernel.v1.proxy.register`: bind a route to that port lease (registered with `ready=false`).
5. Readiness probe: TCP-connect to the loopback port (with an optional health_path HTTP probe). The route is flipped to `ready=true` and success returned only if the probe passes within a bounded timeout.

If any step fails, the broker rolls back in reverse: unregister proxy, stop the just-started container, release the port lease. Because orchestration is host-side, closing the browser tab does not leave orphan containers or port leases.

Stop deployment (`POST /host/v1/deploy/stop`) finds and stops the container by Docker label (`route_id`). It does not rely on a container id remembered by the browser, and does not stop unknown containers based on a matching `port_name`.

## Build & Deploy flow

Build & Deploy uses `POST /host/v1/build-deploy`. By default it returns immediately with `job_id`, a status URL, and an SSE events URL. Long-running work stays in the host broker:

1. Validate source URL, strategy, runtime env, runtime mounts, and user approvals.
2. Clone into the project workspace through `git-tools-lab`.
3. If strategy is `nixpacks`, generate Dockerfile / context first.
4. Call `official/docker-runtime-lab/build_image` and label the image with `project_id`, `build_id`, `source_commit`, `strategy`, and `build_descriptor_hash`.
5. Enter the normal deploy chain: port lease → container start → proxy register → readiness probe.
6. On success, the route becomes `ready=true`; on failure or cancellation, acquired resources roll back in reverse.

Jobs are in-memory only and exist for UI progress plus a bounded log ring. After host restart, job logs may be gone; actual deployment state is recovered from Docker labels, port/proxy event replay, and restart reconciliation.

## `project.start` does not deploy

`kernel.v1.project.start` remains a project state machine: open or reuse a project session, mark Running, and return `session_id`. It does not start a process, allocate a port, or register a proxy.

Deployment is a separate, explicit host-broker action. This keeps “open project UI” and “run an external service” visibly separate.

## Red lines

- Deny-all by default. No profile opt-in means no real local process start.
- Ports bind to loopback only.
- Proxy upstreams must reference active port leases, and `port_name` must match.
- The reverse proxy does not follow upstream redirects and strips `Set-Cookie`, `Location`, CORS, and other dangerous response headers.
- Prebuilt-image deployment does not accept env / volume / secret fields. Source Build & Deploy accepts only explicitly approved runtime env / volume fields; raw runtime secrets are injected only inside the host-private runner, and build-time secrets are not supported yet.
- Docker is implemented by an ordinary capability package. No official fast path.

## Next

- Native execution remains trusted/dev-oriented. It is not a full OS sandbox.
- **Auto-restart** is not implemented and is a separate future phase. Health supervision only monitors, flips readiness, and audits; it does not re-deploy a crashed container. Auto-restart first requires durable "deploy intent" (image, etc.) modeled in host-plane terms without leaking Docker semantics into the kernel proxy / port records — a design done on its own.
- Remote targets and multi-client public exposure are not implemented; ports bind to loopback only.
- Docker descriptors still lack pull progress and long-term log archival.
- External project wizards can generate deployment descriptors later, but deployment must remain an explicit user action.
