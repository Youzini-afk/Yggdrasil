# Self-Hosted Deployment Hub Plan

> [English](./DEPLOYMENT_HUB_PLAN.en.md) · [中文](./DEPLOYMENT_HUB_PLAN.md)
>
> Temporary plan document. Fold into long-term docs and delete this file once landed.

## Background and motivation

The platform vision is unchanged: content-free kernel + contract, capability packages, official-and-third-party equal, composition. What this phase adjusts is the **evolution focus**: the current priority is to evolve Yggdrasil into a **self-hostable AI/agent project deployment & management hub** — helping users deploy, run, and manage community projects (Docker first, native later), accessed from desktop / web / mobile clients.

Why this first:

- The capability-package ecosystem needs users before it can spin up; users will not come for "a platform with no ecosystem yet". Win users with "immediately useful", then grow the ecosystem.
- "Deployment assistance" is not a new subsystem; it is **another platform capability**, expressed as capability packages, running on the content-free kernel, equal to all others.
- A deployed project is just a Path B self-contained application — already an equal citizen of the platform.

This phase **does not change the README / vision positioning**. Play-creation integration and the capability ecosystem remain the destination; the deployment hub is the on-ramp.

## Current state (code-audit conclusions)

- `kernel.v1.project.start` is a pure state machine: auth-check → open session → emit event → mark Running. It spawns no process, allocates no port, returns no URL, does no health check, does no proxy.
- `OutboundExecutor` is a mature, copyable template: trait + `{DenyAll, Fake, LiveHttp}` config enum + `RuntimeConfig` injection + host-profile selection + dispatch + audit + `secret_ref` injection.
- `subprocess.rs` can spawn / kill / timeout, but is JSON-RPC-stdio only, with no port / HTTP concept.
- Reverse proxy: zero implementation in the whole repo. This is the biggest gap.
- The External Operating Plane packages (intake / workspace / install / git-tools / integrity) are all plan-only / fake-executor.

## Kernel boundary decision (settled: A)

The kernel gains a set of **generic "controlled local execution + port + route" primitives**, isomorphic to the existing outbound HTTP executor. The kernel never understands "Docker / deployment / Tavern"; it only provides a generic, audited, policy-gated local-execution and port/route transport. Docker / native / deployment logic all live in ordinary capability packages.

The reverse-proxy data plane is **implemented in-house** (not orchestrating Traefik), placed in `ygg-service`, because browser traffic does not go through capability invoke, and a self-contained single process is friendlier to desktop / cloud / mobile thin clients.

## Architecture

```
   Desktop / Web / Mobile thin clients
            │ all connect to
            ▼
   Yggdrasil control plane (content-free kernel)
   ├ kernel.v1.exec.*    generic process management (DenyAll / Fake / LiveLocal)
   ├ kernel.v1.port.*    port lease (loopback only)
   ├ kernel.v1.proxy.*   route registration (upstream = port lease)
   ├ kernel.v1.target.*  execution target (local / remote / tunnel)
   └ ygg-service reverse-proxy data plane (virtual-host first, path-prefix fallback)
            │ drives
            ▼
   Capability package layer (official and third-party equal)
   ├ official/docker-runtime-lab   (uses bollard)
   ├ official/native-runtime-lab   (later, dangerous, trusted-only)
   ├ official/target-registry-lab
   └ official/deployment-plan-lab
            │
            ▼
   Local Docker / Remote Docker / Mobile native — run any community project
```

## Kernel primitive spec

Follows the `OutboundExecutor` pattern. New `KernelMethod` variants:

```
# Target domain
kernel.v1.target.list / status / register / unregister

# Exec domain
kernel.v1.exec.start / stop / status / logs / list

# Port domain
kernel.v1.port.lease / release / status / list

# Route domain
kernel.v1.proxy.register / unregister / status / list
```

Executor config (mirrors `OutboundExecutorConfig`):

```rust
pub enum LocalExecExecutorConfig {
    DenyAll,                              // default fail-closed
    Custom(Arc<dyn LocalExecExecutor>),   // Fake
    LiveLocal(LiveLocalExecConfig),       // opt-in
}
```

Injected into `RuntimeConfig`, selected by host profile, policy-checked before dispatch, audited with redaction.

Key constraints:

- `ExecCommand` is `{program, args}`, **no shell string**.
- Ports in v1 are **loopback bind only**.
- Proxy upstream must reference a kernel-issued port lease, not an arbitrary URL (prevents open relay).
- env supports `Literal / SecretRef / PortRef`; `secret_ref` must be declared in the manifest.

New events (redacted; never persist raw env / logs / body / secret):

```
kernel/v1/exec.request / denied / started / stopped / completed / failed
kernel/v1/port.leased / released / denied
kernel/v1/proxy.registered / unregistered / denied / access.summary
```

## Reverse-proxy design

- Data plane in `ygg-service`, inserting proxy routing before the SPA fallback.
- Route registry + policy / audit authority in the runtime.
- **Virtual-host first**: `<route_id>.apps.<host>` / `<route_id>.localhost:<port>`, so a community app believes it owns `/` (so `fetch('/api/*')`, cookies, WebSocket do not break).
- Path-prefix `/_ygg/app/<route_id>/` is fallback only, for apps that support a base path.
- Supports HTTP/1.1 + WebSocket upgrade + streaming + body cap + idle timeout + per-route auth.
- Browser iframe uses a one-time launch-token bootstrap → sets a route-scoped cookie → redirects to a clean URL; strips Ygg's own auth header / cookie before forwarding upstream.
- Never forward the host `access_token` to a proxied app.

## Execution target model

First-class but content-free:

```rust
pub struct ExecutionTargetDescriptor {
    target_id, display_name,
    reachability: { LocalHost, RemoteAgent, ReverseTunnel },
    capabilities: [ LocalExec, PortLease, HttpProxyUpstream, WebSocketProxyUpstream ],
    status, registered_by_package_id, metadata,
}
```

The kernel does not know `DockerTarget`. Docker is capability-package / metadata vocabulary. A running project is exposed to clients via a derived `ProjectRunInstance` view (run_id / project_id / target_id / exec_id / port_lease_ids / proxy_route_ids / status / urls).

## Security posture (redlines)

Running an arbitrary community project = running arbitrary code; treat everything as hostile unless isolated.

1. Deny all local execution / port lease / route registration by default.
2. Ports bind loopback only.
3. Exposed only through the authenticated ygg-service reverse proxy.
4. No public exposure without explicit host-admin approval.
5. No raw secret in audit / logs / env persistence / crash records.
6. No shell command string, argv only.
7. Proxy upstream must reference a port lease.
8. Docker default-deny: `--privileged`, `--network host`, mounting `/`, mounting docker.sock, mounting credential dirs, running as root, exposing `0.0.0.0` directly, `latest` without digest pin. Dangerous options only via host-admin explicit override + loud audit.
9. Native execution is unsafe for untrusted community projects; only trusted / dev mode until real OS sandboxing exists.

## Phases (validate + commit + push per phase)

### Phase 1 — Kernel primitive skeleton (deny / fake only)
- Add `kernel.v1.target.* / exec.* / port.* / proxy.*` protocol methods + schema.
- `DenyAllLocalExecExecutor` + `FakeLocalExecExecutor` + in-memory registries.
- Audit events, permission declarations, host-profile config.
- No real process spawn.
- Acceptance: a test package gets deterministic fake handles; denied calls never reach the executor; audit carries no raw secret / env / logs.

### Phase 2 — ygg-service generic reverse proxy (virtual-host first)
- Insert dynamic proxy routing before the SPA fallback, looking up routes from the runtime proxy registry.
- Virtual-host mode + path-prefix fallback + WebSocket + launch token + header/cookie stripping + route disable/expiry.
- Acceptance: a fake upstream can be opened from an iframe; `fetch('/api/*')` works in virtual-host mode; WebSocket echo passes; a proxied app cannot call `/kernel/v1/*` with inherited credentials.

### Phase 3 — LiveLocal exec + Docker-first package
- `LiveLocalExecExecutor` (`tokio::process::Command` + long-lived process table + stdout/stderr ring buffer + stop/kill timeout + readiness probe).
- `official/docker-runtime-lab` (uses bollard).
- Smallest proof milestone: deploy a Docker community web project → lease loopback port → register proxy → open from web client → view logs → stop → full audit. Green a tiny known Docker web app first, then dogfood YdlTavern / a community Tavern.

### Phase 4 — Project run UX + target model
- Target list/status UI, project run cards, start/stop/log/open actions, `ProjectRunInstance` projection, route health, per-project secrets UI, approval prompts for local-exec / port-expose / docker-run.

### Phase 5 — intake → install → deploy pipeline
- Wire project intake → install-lab → integrity-lab → deployment-plan-lab → docker-runtime-lab.
- Acceptance: a real non-Yggdrasil external project can be installed and deployed without hand-written one-off code; plans are inspectable before execution; permissions/proposals are readable.

### Phase 6 — Doc convergence
- Delete this temporary plan, update long-term docs (NEXT_STEPS / ALPHA_STATUS / relevant guides).
- Remote targets / native / mobile thin client listed as follow-up, not forced complete this round.

## Out of scope (this round)
- Orchestrating Traefik / Caddy.
- Promoting native execution as a "safe" path.
- Full remote target agent / mTLS / reverse-tunnel implementation (seam only).
- Mobile native execution (mobile is a remote thin client first).
- Introducing any docker / container / deploy / tavern semantics into the kernel.
