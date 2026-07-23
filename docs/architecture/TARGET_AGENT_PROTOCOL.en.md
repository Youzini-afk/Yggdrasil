# Target Agent Protocol

> [English](./TARGET_AGENT_PROTOCOL.en.md) · [中文](./TARGET_AGENT_PROTOCOL.md)

Status: **implementation contract; Phase 4 in progress**. A Target Agent is a remote execution adapter for the Host Control Plane. It is not a remote package, general SSH shell, second Host, or application identity provider.

## Three remote boundaries

| Boundary | Subject | Purpose | Trust |
|---|---|---|---|
| Remote Host client | Human device, Web/PWA, CLI | Control one Host | root or project-scoped device grant |
| Remote Target Agent | Managed execution node | Execute deployment/verifier operations and report truth | target identity + operation authority |
| Remote package entry | Capability-provider service | Serve package invoke/stream | workload identity + attenuated capability |

They never share bearer credentials, lifecycle, or implicit authority.

## Roles

- **Controller:** owner of desired state, operations, leases, policy, and audit.
- **Agent:** executes typed operations locally, persists an operation ledger, and observes entities.
- **Target driver:** transport-neutral Controller seam; local and remote implementations pass the same conformance.
- **Artifact service:** authorized, digest-verified content transport.
- **Ingress/tunnel adapter:** connects Host routes to agent loopback without treating arbitrary remote IPs as local leases.

An Agent does not own the project catalog, grant registry, user sessions, package marketplace, or final deployment intent.

## Target lifecycle

```text
ExecutionTarget
  id / display_name
  reachability: local | direct | reverse_tunnel
  identity_ref
  protocol_versions[]
  declared_capabilities[] / effective_capabilities[]
  labels{}
  status / last_seen_at?
  enrolled_at / revoked_at?
```

States include Enrolling, Available, Degraded, Offline, Draining, Incompatible, and Revoked. Effective capability is the intersection of declaration, Host policy, and verified probing. Caller-submitted JSON cannot make a target Available.

## Enrollment and identity

1. root or target-manage authority creates a short-lived single-use challenge;
2. the Agent generates its key and submits challenge, public key, version, and capabilities;
3. the Host verifies it and registers a Host-audience target identity;
4. sessions use mTLS or equivalent mutual authentication with rotation;
5. revocation rejects new sessions and operations; drain/revoke policy handles existing workloads.

The journal stores public identity, credential digest/serial, state, and audit references, never the agent private key. A Host-managed CA can implement v1 while preserving future SPIFFE integration.

### Implemented identity/observation and typed-worker slices

The `target-agent.v1` identity and observation control plane exposes:

| Caller | Route | Authority and purpose |
|---|---|---|
| Host client | `POST /host/v1/targets/{target_id}/enrollments` | `deploy` scope plus target selector; creates a single-use challenge with a maximum 15-minute lifetime |
| Agent | `POST /target-agent/v1/enroll` | Consumes the challenge, negotiates version/capabilities, and receives a Host-generated bootstrap target credential once |
| Agent | `POST /target-agent/v1/heartbeat` | Separate `YggTarget` credential; refreshes observation and 45-second liveness |
| Host client | `GET /host/v1/targets/{target_id}/observe` | `observe` scope plus target selector; returns declarations, effective capabilities, epochs, and observed summary |
| Host client | `POST /host/v1/targets/{target_id}/revoke` | `deploy` scope plus target selector; revokes identity and advances both lease and policy epochs |

Enrollment tokens and agent credentials enter the `host_control_target_agents` journal only as domain-separated SHA-256 digests. Challenges are single-use; after restart every non-revoked remote target first returns to `Offline`; an old credential or epoch cannot restore availability. The compatibility names `kernel.v1.target.register/unregister` now fail closed, so caller JSON cannot bypass enrollment and create an `Available` target.

Phase 4B adds these typed-worker routes without adding a general command surface:

| Caller | Route | Authority and purpose |
|---|---|---|
| Host client | `POST/GET /host/v1/targets/{target_id}/operations` | `deploy`/`observe` plus target and project selectors; create or list typed operations |
| Host client | `GET /host/v1/targets/{target_id}/operations/{operation_id}` | Read a project-scoped durable operation and receipt |
| Agent | `GET /target-agent/v1/operations/next` | Return pending work only to the live target with matching epochs |
| Agent | `POST /target-agent/v1/operations/{operation_id}/progress` | Persist accepted/running; the first random `execution_id` owns execution |
| Agent | `POST /target-agent/v1/operations/{operation_id}/receipt` | Accept a terminal receipt only when authority, execution owner, and request digest match |
| Agent | `GET /target-agent/v1/operations/{operation_id}/artifacts/{digest}` | Stream only a digest explicitly authorized by that accepted/running operation |

The Host `host_control_target_operations` journal and Agent SQLite ledger both use expected-sequence CAS. The Agent persists request/authority digests before acknowledging acceptance and persists the terminal receipt before posting it. A process lock protects one data directory, while a copied credential with another ledger cannot take over an already-bound `execution_id`. Executable types now include `artifact.materialize/release`, `health.probe`, declarative `verifier.run(artifact_integrity)`, and `deployment.apply/observe/drain/stop`; unknown types have no shell fallback. Downloads use a digest-derived partial path and enter the local CAS only after full SHA-256 and size verification.

Revoke fails closed for new work and new accepted/running transitions. The linearization boundary is the Host's durable acknowledgement of `Running`: a revoke/offline/stale epoch observed before it prevents execution; after it, the current idempotent step may finish or replay its receipt but cannot acquire new work. Revoke does not pretend to atomically roll back a target-local effect: `deployment.drain` performs a bounded graceful stop and retains the container, while `deployment.stop` removes it and force-removes only when explicitly requested.

Operation authority binds target, operation, step, project, effect, artifacts, lease/policy epochs, expiry, nonce, and request digest. Remote Agent authority is MACed with the epoch-scoped, domain-separated enrollment credential digest, and the Agent independently recomputes that MAC from the credential it received once. A local-driver journal record uses a stable domain-separated key confined to the Host and does not treat that key as a network identity. The native client disables redirects, requires HTTPS for a remote Host, never persists the credential in config or ledger, and reads it only from `YGG_TARGET_AGENT_CREDENTIAL`; loopback HTTP is confined to the same machine.

Phase 4C now routes local versus agent drivers from `ExecutionTargetReachability`; no caller-provided network address can act as a driver fallback. Local and Agent deployment operations share one typed Docker driver: non-privileged bridge networking, `127.0.0.1` binding only, no command/env/mount inputs, and idempotent lookup through target/project/deployment/route/lease/operation ownership labels. The `apply` receipt returns Docker's actual loopback port. An effect that was issued but cannot be confirmed becomes `outcome_unknown` rather than a false failure; Host startup durably resolves interrupted local Accepted/Running records the same way. Host actual-port lease registration, authenticated tunnel/private preview remain later slices, and the Host loopback upstream boundary is unchanged.

## Transport session

The protocol runs over an authenticated bidirectional session independent of connection direction:

```text
Hello(target_id, identity, versions, nonce)
Welcome(host_id, selected_version, session_id, policy_epoch)
Heartbeat(observed_summary, receipt_cursor)
OperationRequest(operation, step, lease_epoch, authority, body)
OperationAccepted | OperationRejected
OperationProgress(sequence, diagnostic_refs)
OperationReceipt(terminal result)
ObserveRequest / ObserveSnapshot
CancelRequest / CancelReceipt
ArtifactRequest / ArtifactChunk / ArtifactReceipt
```

V1 selects one transport. Direct mTLS HTTP/2 and reverse tunnel may later be connection adapters without changing operation semantics. Reconnect resumes from a durable receipt cursor.

## Typed operations

Agents accept only public versioned policy-decidable operations: artifact materialize/release, deployment apply/observe/stop/drain, health probe, logs read/follow, actual port reserve/release, tunnel open/close, and declarative verifier run.

There is no `shell(command: string)`. Process execution, when needed, constrains program, args, cwd, env, network, mounts, resources, and output under both target policy and operation authority. Unknown operations and fields fail closed.

## Operation authority and fencing

```text
OperationAuthority
  audience_target_id
  operation_id / step_id
  project_resource_ref
  allowed_effect
  artifact_digests[] / secret_envelope_refs[]
  lease_epoch
  issued_at / expires_at
  nonce / authority_digest
```

It is bound to one target, operation, step, and effect. The Agent verifies Host identity, audience, expiry, policy epoch, and lease epoch. `(operation_id, step_id, request_digest)` is idempotent; a different digest for the same step conflicts. Older epochs are rejected even with valid signatures. V1 may encode this as an mTLS-session-bound, signed, or MACed short-lived token without weakening semantics.

## Agent ledger

The Agent persists request digest and epoch before acknowledging acceptance, and persists a receipt before acknowledging terminal state:

```text
accepted -> running -> succeeded | failed | cancelled
                     -> outcome_unknown
```

After restart it reconciles the ledger with ownership-labelled local entities. Uncertain effects remain outcome_unknown.

## Artifacts and secrets

Artifacts are content addressed, chunked, resumable under a lease, fully digest-verified before activation, authorized per operation, and retained by active/previous/in-flight/pinned reachability. Provenance, signature, and media type travel with the descriptor.

Journals and artifacts carry only `secret_ref`. The Host creates a short-lived envelope for one target identity and operation; the Agent decrypts as late as possible into tmpfs/restricted executor facilities and destroys temporary material after terminal state. There is no general secret list/get API.

## Network and ingress

The Host loopback-only upstream rule remains a security boundary. Remote routing uses an authenticated target tunnel:

1. Agent actually reserves a target-loopback port;
2. Controller registers a target-aware route;
3. Host proxy opens a stream through the authenticated tunnel;
4. Agent validates route, lease, generation, and epoch before dialing loopback;
5. Host route policy continues to decide public versus Host-authenticated access.

Target-side public ingress, ACME, and edge identity are later adapters, not a hidden relaxation in v1.

## Failure behavior

| Condition | Behavior |
|---|---|
| Heartbeat timeout | Offline; workloads are not assumed absent |
| Reconnect | Revalidate identity/epoch and resume receipt cursor |
| Host crashes after request | Observe/ledger lookup before retry |
| Agent crashes after start | Reconcile ledger and ownership labels |
| Network partition | Expiring authority and fencing prevent split ownership |
| Target revoked | Reject new work; apply explicit drain policy |
| Artifact corruption | Delete partial data and terminally fail the step |
| Tunnel loss | Route becomes unready while intent remains |
| Version mismatch | Incompatible; never execute unknown semantics |

## Delivery slices

1. Identity and observation: durable registry, enrollment, heartbeat, negotiation, observe.
2. Typed verifier worker: artifact transfer, declarative verifier, receipts/logs.
3. Private deployment preview: deployment/port/tunnel operations and Host-authenticated route.
4. Public deployment through an already-public Host, followed later by target-edge design.

Initial placement is explicit. No automatic scheduler, multi-Host leader election, or secret federation.

## Completion gate

- local and remote target implementations produce equivalent operation states and receipts;
- duplicate, out-of-order, expired, and stale-epoch requests are deterministic;
- crashes/disconnects at every step do not duplicate workloads;
- revoke, drain, reconnect, corruption, and version mismatch have coverage;
- no general shell, device credential, or cross-project artifact/secret access exists;
- remote routing does not rely on arbitrary network upstreams.
