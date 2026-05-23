# Round 10A.3 — End-to-End Real Path Plan

> Temporary planning document. Removed at Wave 4 once docs converge.

## Mission

Connect the previously assembled pieces into a real working path so a user can:
1. Open Yggdrasil
2. See YdlTavern card on Home
3. Click Play (project starts, surface mounts)
4. Configure OPENAI_API_KEY in API Connections drawer
5. Type a message + Send
6. Get a real streaming response from OpenAI/Anthropic/etc.

The plumbing exists (Round 10A.1 secret store, 10A.2 project model, engine.model.live_call, outbound.execute LiveHttp). The gap is the surface↔engine wiring at the top, the bundle resolver at the middle, and a few session metadata details.

Plus a tech-debt cleanup (huggingface-fetcher).

## Audit findings

```text
huggingface-fetcher (YdlTavern):
  All 7 failures are deterministic TypeError:
  test reads kernel.v1.calls but makeMockKernel returns { calls, sendKernelRequest }
  Cleanest fix: standardize mock to expose kernel.v1.calls (or update tests to
  use kernel.calls). 30 minutes of work.

surface mounting (Yggdrasil clients/web):
  iframe pipeline works (postMessage protocol, sandbox, ESM dynamic import)
  YdlTavern surface ESM bundle exists at packages/ydltavern-surface/dist/bundle.mjs
  Vite middleware serves /surface-bundles/ydltavern/* from sibling repo
  Hardcoded demo resolver in clients/web/src/surfaces/bundle-resolver.ts
  Need: replace hardcoded map with metadata-driven path that uses installed
  project descriptors at ~/.yggdrasil/projects/<id>/ (and fallback to dev path)

real model e2e (cross-repo):
  Surface SendForm → TavernProvider.sendMessage only appends to local state
  Engine model.live_call + .stream registered correctly
  Engine reverse-calls kernel.v1.outbound.execute with secret_headers
  Host dispatch resolves secrets via session_id → session.metadata.project_id
  LiveHttpOutboundExecutor injects headers, real HTTPS works
  
  Gap 1: SendForm/TavernProvider doesn't invoke engine capability
  Gap 2: session.metadata.project_id is not auto-set when project starts
  Gap 3: surface doesn't have a session_id to attach to RPC calls
  Gap 4: engine manifest doesn't declare secret_ref:project:* yet
```

## Wave structure

```
Wave 1: huggingface-fetcher tech debt              (~half day)
Wave 2: real surface bundle resolution              (~1-2 days)
Wave 3: surface → engine → real HTTPS wiring        (~2-3 days)
Wave 4: docs convergence + delete plan              (~half day)
```

Total: ~4-6 days.

## Wave 1 — huggingface-fetcher fix

**Goal**: 7 pre-existing failures gone. ydltavern-engine-core test count comes up to 100% pass.

### Changes

In `/workspace/Yggdrasil/YdlTavern/packages/ydltavern-engine-core/test/huggingface-fetcher.test.ts` (or similar):

The `makeMockKernel()` helper returns `{ calls, sendKernelRequest }`, but failing tests read `kernel.v1.calls`. Two fix options:

**Option A**: Update mock to match what tests expect:
```ts
function makeMockKernel() {
  const calls: KernelCall[] = [];
  return {
    calls,                      // existing
    sendKernelRequest: ...,     // existing
    v1: {                       // new shim for tests that use kernel.v1.calls
      calls,
    },
  };
}
```

**Option B**: Update all 7 tests to use `kernel.calls` instead of `kernel.v1.calls`.

Pick whichever keeps the diff minimal. Option B is 7 simple search-and-replace changes; Option A is 3 new lines and matches test expectations.

If both styles are used in other test files, prefer the dominant style.

Also verify the implementation file doesn't need changes — the audit said the implementation is fine.

### Validation

```bash
cd /workspace/Yggdrasil/YdlTavern
npm test --prefix packages/ydltavern-engine-core 2>&1 | tail -10
# All 7 should now pass; total count grows from ~85 pass / 7 fail to ~92 pass / 0 fail
```

## Wave 2 — Real surface bundle resolution

**Goal**: clients/web's `resolveSurfaceBundle()` reads project descriptor metadata to find bundle URLs, instead of a hardcoded map.

### Architecture decision

Two paths to surface bundle URLs:

1. **Development mode**: bundle served from sibling YdlTavern repo via Vite middleware (current behavior). Keep as fallback.
2. **Installed mode**: bundle served from `~/.yggdrasil/projects/<id>/dist/` after install copies it.

For Wave 2, support both. New `kernel.v1.surface.resolve_bundle` method returns the bundle URL given a surface_id, looking up:
1. Active project descriptors → if surface_id matches a project's surface contributions, return the bundle URL relative to the project's data dir
2. Sibling repo paths (dev mode override via env or config) → return `/surface-bundles/<project>/bundle.mjs`
3. Otherwise fail with `surface_not_found`

### Changes

#### Yggdrasil side

1. **New protocol method** `kernel.v1.surface.resolve_bundle`:
   - Input: `{ "surface_id": String }`
   - Output: `{ "surface_id": String, "bundle_url": String, "stylesheets": [String], "export_name": String, "project_id": String?, "wrapper_class": String? }`

   Schema in `docs/spec/v1/schemas/methods/`.

2. **Implementation** in `crates/ygg-runtime/src/runtime/protocol_dispatch.rs`:
   ```rust
   async fn dispatch_surface_resolve_bundle(&self, _ctx: &ProtocolContext, params: Value) -> Result<Value> {
       let surface_id = params.get("surface_id")
           .and_then(Value::as_str)
           .ok_or_else(|| anyhow::anyhow!("surface_id required"))?;
       
       // Strategy 1: walk project registry, look at each project's package surface contributions
       for entry in self.config.project_registry.list() {
           // Check if any of this project's packages contributes this surface_id
           // If yes: build URL relative to project data dir
           // ...
       }
       
       // Strategy 2: dev fallback via host-config-provided sibling path mapping
       // (HostProfile.surface_dev_paths: HashMap<String, String> mapping prefix → filesystem dir)
       
       anyhow::bail!("surface_not_found: {}", surface_id);
   }
   ```

3. **HostProfile** new optional section for dev-mode bundle paths:
   ```yaml
   surface_dev_paths:
     ydltavern: /workspace/Yggdrasil/YdlTavern/packages/ydltavern-surface/dist
   ```
   
   When `surface_id` starts with `ydltavern/`, look up the prefix and serve from that path.

4. **Vite dev middleware** in `clients/web/vite.config.ts` (or server config):
   - On request to `/surface-bundles/<prefix>/<file>`, if a surface_dev_paths mapping exists, serve from that path.
   - Else 404.
   
   Keep the existing ydltavern-st-compat-server middleware working; just generalize the bundle path.

#### clients/web side

1. **Replace `resolveSurfaceBundle()`** in `clients/web/src/surfaces/bundle-resolver.ts`:
   ```ts
   import { YggProtocolClient } from '../protocol/client.js';
   
   export interface SurfaceBundle {
     surfaceId: string;
     bundleUrl: string;
     stylesheets: string[];
     exportName: string;
     wrapperClass?: string;
     projectId?: string;
   }
   
   export async function resolveSurfaceBundle(client: YggProtocolClient, surfaceId: string): Promise<SurfaceBundle> {
     const result = await client.invoke('kernel.v1.surface.resolve_bundle', { surface_id: surfaceId });
     return {
       surfaceId: (result as any).surface_id,
       bundleUrl: (result as any).bundle_url,
       stylesheets: (result as any).stylesheets ?? [],
       exportName: (result as any).export_name,
       wrapperClass: (result as any).wrapper_class,
       projectId: (result as any).project_id,
     };
   }
   ```
   
   Update all call sites in main.ts.

2. **Project play flow** in main.ts:
   When user clicks Play on a project:
   1. `client.startProject(projectId)` (already wired in Wave 4 of 10A.2)
   2. `client.getProject(projectId)` to read `entry_surface_id`
   3. `resolveSurfaceBundle(client, project.entry_surface_id)`
   4. `mountSurface({ ..., bundleUrl, exportName, hostBridge: { callRpc: ... } })`

### Validation

- Conformance: 1-2 new cases for `kernel.v1.surface.resolve_bundle`
- Schema count grows from 114 to 115
- Manual smoke (in dev mode):
  ```bash
  cd /workspace/Yggdrasil/Yggdrasil/clients/web
  npm run dev
  # In another terminal:
  cd /workspace/Yggdrasil/YdlTavern && npm run build --prefix packages/ydltavern-surface
  
  # Browse to localhost:5173, register YdlTavern as project (if needed), click Play
  # Surface should mount, render YdlTavern UI, kernel events should flow
  ```

If full smoke isn't feasible in sandbox, the conformance case for resolve_bundle is sufficient evidence of correctness.

## Wave 3 — Real model end-to-end

**Goal**: User clicks Send in YdlTavern surface → real OpenAI/Anthropic API call → streamed response.

### Wave 3.1 — session.metadata.project_id auto-population

When a project is started via `kernel.v1.project.start`, the dispatch handler should:
1. Open a host-admin kernel session for the project (or reuse one)
2. Set `session.metadata.project_id = project_id`
3. Return the session_id alongside state info

This way, when the surface gets the response from `kernel.v1.project.start`, it knows which session_id to attach to subsequent RPC calls. Project-scoped secrets resolve correctly because the session has metadata.project_id.

Update:
- `crates/ygg-runtime/src/runtime/protocol_dispatch.rs::dispatch_project_start` adds session.open + metadata
- `kernel.v1.project.start` output schema gains `session_id: String`
- `kernel.v1.project.get` output schema also exposes the running session_id

### Wave 3.2 — Surface session_id passing

In `clients/web/src/main.ts`:
- After `client.startProject(projectId)` returns, capture `session_id`
- Pass session_id into the surface via `initialProps`
- Store it for forwarded RPC calls

In the surface host bridge:
- All `client.invoke(...)` calls from the surface use this session_id by default in the RPC request top-level

In `packages/ydltavern-surface/src/host-rpc/index.ts`:
- `callHostRpc` accepts an optional session_id (or reads from initial props)
- Propagates it as the top-level `session_id` field in the RPC message

### Wave 3.3 — Surface SendForm wiring

In `packages/ydltavern-surface/src/app/TavernProvider.tsx::sendMessage()`:
Currently appends user message to local chat state. Extend:
1. Append user message (existing)
2. Construct model.live_call.stream invocation:
   ```ts
   const result = await invokeCapability('ydltavern/engine/model.live_call.stream', {
     source: connectionSettings.provider,  // openai/anthropic/...
     model: connectionSettings.model,
     base_url: connectionSettings.baseUrl,
     secret_ref: connectionSettings.secretRef,  // secret_ref:store:OPENAI_API_KEY etc.
     messages: chat.map(m => ({ role: m.role, content: m.content })),
     stream: true,
   });
   ```
3. As stream chunks arrive (kernel.v1.capability.stream events), append assistant message progressively
4. On stream end, finalize the assistant message

The streaming consumer pattern:
- Use `kernel.v1.capability.stream` if available, or fall back to non-streaming `kernel.v1.capability.invoke` for first iteration
- The engine's `model.live_call.stream` capability already exists (per audit)

### Wave 3.4 — Engine manifest project secrets

In `packages/ydltavern-engine/manifest.yaml`:
Add `secret_ref:project:*` patterns alongside existing `secret_ref:env:*` and `secret_ref:store:*`:

```yaml
permissions:
  secret_refs:
    - secret_ref:env:OPENAI_API_KEY
    - secret_ref:env:ANTHROPIC_API_KEY
    - secret_ref:store:OPENAI_API_KEY
    - secret_ref:store:ANTHROPIC_API_KEY
    - secret_ref:project:OPENAI_API_KEY    # NEW
    - secret_ref:project:ANTHROPIC_API_KEY # NEW
    # ... etc for all supported providers
```

### Wave 3.5 — Conformance

- `e2e.surface_send_invokes_engine_capability` — surface mock + engine fake → message flows through
- `e2e.session_metadata_project_id_set_on_start` — project start sets session metadata
- `e2e.engine_manifest_declares_project_secret_refs` — manifest validation
- `e2e.streaming_response_arrives_in_surface` — fake stream from engine → surface gets chunks

### Validation

- ydltavern-surface tests pass (with new SendForm flow tests)
- ydltavern-engine tests pass
- Yggdrasil conformance grows by ~4 cases

For real network smoke:
- Set `OPENAI_API_KEY` env var or store secret via UI
- Start host serve with live HTTPS executor + secret resolver
- Open browser, navigate to YdlTavern surface
- Send a message
- Get a real response

If sandbox can't reach OpenAI:
- Use FakeOutboundExecutor with deterministic responses
- Verify the call shape matches what would be sent live
- Document opt-in flag (`YGG_LIVE_E2E_TESTS=1`) for users with API keys

## Wave 4 — Docs convergence + delete plan

### Changes

1. **Update PROJECT_MODEL guide** with Play→start→surface mount→session flow
2. **Update SECRET_MANAGEMENT** with project-scoped flow example for YdlTavern
3. **New `docs/guides/REAL_MODEL_END_TO_END.{md,en.md}`** describing:
   - The full chain (surface → engine → outbound → API)
   - How to set up live calls with secret store
   - Troubleshooting (auth errors, network, etc.)
4. **Update KERNEL_V1_CONTRACT** with new surface.resolve_bundle method
5. **Update ALPHA_STATUS** with Round 10A.3 section
6. **Update CONFORMANCE_MATRIX** with new cases
7. **Update NEXT_STEPS** marking 10A.3 complete; outline Round 10B (WIT/WASM)
8. **Update README** (both repos): note real model calls work end-to-end
9. **Delete `docs/E2E_REAL_PATH_PLAN.md`** (this file)

## Wave plan

```
Wave 1: huggingface-fetcher fix      (independent, YdlTavern, ~30min)
Wave 2: surface bundle resolution    (Yggdrasil + clients/web, ~1d)
Wave 3.1-3.4: real model wiring     (cross-repo, ~2d, must follow Wave 2)
Wave 3.5: conformance               (~30min)
Wave 4: docs                         (~30min)
```

Wave 1 and Wave 2 can be fully parallel.

## Push cadence

```
Plan written + pushed                           (now)
Wave 1 done + pushed                            (~30min after start)
Wave 2 done + pushed                            (~1d)
Wave 3 done + pushed                            (~2d after Wave 2)
Wave 4 done + pushed (deletes this plan)        (~half day after Wave 3)
Final report to user                            (after Wave 4)
```

## Constraints

- AGPL-3.0
- Bilingual docs
- No new kernel ontology beyond `kernel.v1.surface.resolve_bundle` (data-only)
- Existing tests stay passing
- secret_ref:env: / store: / project: behavior unchanged
- Engine manifest changes are additive (more secret_refs, no removals)
- Surface bundle resolution backward compatible: dev path still works

## Out of scope

- Tauri shell + native install distribution (Round 11+)
- yg gc / Sigstore / OS keyring (Round 11+)
- Multi-tenant ProtocolContext.session_id (Round 11+)
- WIT/WASM (Round 10B, separate round)
- v0.1.0 release tag (after Round 10A.3 + 10B)
