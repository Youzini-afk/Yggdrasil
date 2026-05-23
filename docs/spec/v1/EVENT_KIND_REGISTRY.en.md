# Event Kind Registry (v1)

This table lists kernel-reserved `kernel/v1/*` event kinds. Non-kernel packages must write under their own package-id namespace and must not write `kernel/v1/*`.

| Event kind | Payload schema | Writer | Trigger | Status |
|---|---|---|---|---|
| `kernel/v1/session.opened` | [`./schemas/events/kernel__v1__session.opened.schema.json`](./schemas/events/kernel__v1__session.opened.schema.json) | kernel | Session opened | implemented |
| `kernel/v1/session.closed` | [`./schemas/events/kernel__v1__session.closed.schema.json`](./schemas/events/kernel__v1__session.closed.schema.json) | kernel | Session closed | implemented |
| `kernel/v1/session.forked` | [`./schemas/events/kernel__v1__session.forked.schema.json`](./schemas/events/kernel__v1__session.forked.schema.json) | kernel | Session fork creates branch lineage | implemented |
| `kernel/v1/package.loaded` | [`./schemas/events/kernel__v1__package.loaded.schema.json`](./schemas/events/kernel__v1__package.loaded.schema.json) | kernel | Package accepted and registered; payload includes `contract_mode` (`v1` or `none`) | implemented |
| `kernel/v1/package.loading` | [`./schemas/events/kernel__v1__package.loading.schema.json`](./schemas/events/kernel__v1__package.loading.schema.json) | kernel | Package enters loading | implemented |
| `kernel/v1/package.starting` | [`./schemas/events/kernel__v1__package.starting.schema.json`](./schemas/events/kernel__v1__package.starting.schema.json) | kernel | Package process/entry starting | implemented |
| `kernel/v1/package.ready` | [`./schemas/events/kernel__v1__package.ready.schema.json`](./schemas/events/kernel__v1__package.ready.schema.json) | kernel | Package ready after startup | implemented |
| `kernel/v1/package.stopping` | [`./schemas/events/kernel__v1__package.stopping.schema.json`](./schemas/events/kernel__v1__package.stopping.schema.json) | kernel | Package execution stopping | implemented |
| `kernel/v1/package.stopped` | [`./schemas/events/kernel__v1__package.stopped.schema.json`](./schemas/events/kernel__v1__package.stopped.schema.json) | kernel | Package execution stopped | implemented |
| `kernel/v1/package.unloaded` | [`./schemas/events/kernel__v1__package.unloaded.schema.json`](./schemas/events/kernel__v1__package.unloaded.schema.json) | kernel | Package removed from registry | implemented |
| `kernel/v1/package.degraded` | [`./schemas/events/kernel__v1__package.degraded.schema.json`](./schemas/events/kernel__v1__package.degraded.schema.json) | kernel | Execution failure or health loss | implemented |
| `kernel/v1/package.log` | [`./schemas/events/kernel__v1__package.log.schema.json`](./schemas/events/kernel__v1__package.log.schema.json) | kernel | Captured subprocess stderr line | implemented |
| `kernel/v1/asset.put` | [`./schemas/events/kernel__v1__asset.put.schema.json`](./schemas/events/kernel__v1__asset.put.schema.json) | kernel | Opaque asset stored | implemented |
| `kernel/v1/projection.updated` | [`./schemas/events/kernel__v1__projection.updated.schema.json`](./schemas/events/kernel__v1__projection.updated.schema.json) | kernel | Projection state rebuilt/updated | implemented |
| `kernel/v1/proposal.created` | [`./schemas/events/kernel__v1__proposal.created.schema.json`](./schemas/events/kernel__v1__proposal.created.schema.json) | kernel | Proposal created | partial |
| `kernel/v1/proposal.approved` | [`./schemas/events/kernel__v1__proposal.approved.schema.json`](./schemas/events/kernel__v1__proposal.approved.schema.json) | kernel | Proposal approved | partial |
| `kernel/v1/proposal.rejected` | [`./schemas/events/kernel__v1__proposal.rejected.schema.json`](./schemas/events/kernel__v1__proposal.rejected.schema.json) | kernel | Proposal rejected | partial |
| `kernel/v1/proposal.applied` | [`./schemas/events/kernel__v1__proposal.applied.schema.json`](./schemas/events/kernel__v1__proposal.applied.schema.json) | kernel | Proposal applied | partial |
| `kernel/v1/proposal.failed` | [`./schemas/events/kernel__v1__proposal.failed.schema.json`](./schemas/events/kernel__v1__proposal.failed.schema.json) | kernel | Proposal apply failed | partial |
| `kernel/v1/capability.invoked` | [`./schemas/events/kernel__v1__capability.invoked.schema.json`](./schemas/events/kernel__v1__capability.invoked.schema.json) | kernel | Capability invocation started | planned |
| `kernel/v1/capability.completed` | [`./schemas/events/kernel__v1__capability.completed.schema.json`](./schemas/events/kernel__v1__capability.completed.schema.json) | kernel | Capability invocation succeeded | planned |
| `kernel/v1/capability.failed` | [`./schemas/events/kernel__v1__capability.failed.schema.json`](./schemas/events/kernel__v1__capability.failed.schema.json) | kernel | Capability invocation failed | planned |
| `kernel/v1/permission.denied` | [`./schemas/events/kernel__v1__permission.denied.schema.json`](./schemas/events/kernel__v1__permission.denied.schema.json) | kernel | Permission check denied | implemented |
| `kernel/v1/permission.granted` | [`./schemas/events/kernel__v1__permission.granted.schema.json`](./schemas/events/kernel__v1__permission.granted.schema.json) | kernel | Permission grant recorded | implemented |
| `kernel/v1/permission.revoked` | [`./schemas/events/kernel__v1__permission.revoked.schema.json`](./schemas/events/kernel__v1__permission.revoked.schema.json) | kernel | Permission grant revoked | implemented |
| `kernel/v1/error` | [`./schemas/events/kernel__v1__error.schema.json`](./schemas/events/kernel__v1__error.schema.json) | kernel | Structured kernel error | planned |
| `kernel/v1/outbound.request` | [`./schemas/events/kernel__v1__outbound.request.schema.json`](./schemas/events/kernel__v1__outbound.request.schema.json) | kernel | Outbound request allowed/audited | partial |
| `kernel/v1/outbound.denied` | [`./schemas/events/kernel__v1__outbound.denied.schema.json`](./schemas/events/kernel__v1__outbound.denied.schema.json) | kernel | Outbound request denied | partial |
| `kernel/v1/outbound.execute.completed` | [`./schemas/events/kernel__v1__outbound.execute.completed.schema.json`](./schemas/events/kernel__v1__outbound.execute.completed.schema.json) | kernel | Outbound execute completed | implemented |
| `kernel/v1/outbound.stream.completed` | [`./schemas/events/kernel__v1__outbound.stream.completed.schema.json`](./schemas/events/kernel__v1__outbound.stream.completed.schema.json) | kernel | Outbound stream completed | implemented |
| `kernel/v1/git_fetch.requested` | [`./schemas/events/kernel__v1__git_fetch.requested.schema.json`](./schemas/events/kernel__v1__git_fetch.requested.schema.json) | kernel | Git fetch requested | partial |
| `kernel/v1/git_fetch.denied` | [`./schemas/events/kernel__v1__git_fetch.denied.schema.json`](./schemas/events/kernel__v1__git_fetch.denied.schema.json) | kernel | Git fetch denied | partial |
| `kernel/v1/git_fetch.completed` | [`./schemas/events/kernel__v1__git_fetch.completed.schema.json`](./schemas/events/kernel__v1__git_fetch.completed.schema.json) | kernel | Git fetch completed | partial |
| `kernel/v1/git_fetch.failed` | [`./schemas/events/kernel__v1__git_fetch.failed.schema.json`](./schemas/events/kernel__v1__git_fetch.failed.schema.json) | kernel | Git fetch failed | partial |
| `kernel/v1/stream.started` | [`./schemas/events/kernel__v1__stream.started.schema.json`](./schemas/events/kernel__v1__stream.started.schema.json) | kernel | Streaming invocation started | partial |
| `kernel/v1/stream.chunk` | [`./schemas/events/kernel__v1__stream.chunk.schema.json`](./schemas/events/kernel__v1__stream.chunk.schema.json) | kernel | Streaming chunk emitted | partial |
| `kernel/v1/stream.progress` | [`./schemas/events/kernel__v1__stream.progress.schema.json`](./schemas/events/kernel__v1__stream.progress.schema.json) | kernel | Streaming progress emitted | partial |
| `kernel/v1/stream.ended` | [`./schemas/events/kernel__v1__stream.ended.schema.json`](./schemas/events/kernel__v1__stream.ended.schema.json) | kernel | Streaming ended normally | partial |
| `kernel/v1/stream.error` | [`./schemas/events/kernel__v1__stream.error.schema.json`](./schemas/events/kernel__v1__stream.error.schema.json) | kernel | Streaming errored | partial |
| `kernel/v1/stream.cancelled` | [`./schemas/events/kernel__v1__stream.cancelled.schema.json`](./schemas/events/kernel__v1__stream.cancelled.schema.json) | kernel | Streaming cancelled | partial |
| `kernel/v1/stream.timeout` | [`./schemas/events/kernel__v1__stream.timeout.schema.json`](./schemas/events/kernel__v1__stream.timeout.schema.json) | kernel | Streaming timed out | partial |
| `kernel/v1/outbound.websocket.opened` | [`./schemas/events/kernel__v1__outbound.websocket.opened.schema.json`](./schemas/events/kernel__v1__outbound.websocket.opened.schema.json) | kernel | Outbound WebSocket opened | implemented |
| `kernel/v1/outbound.websocket.frame` | [`./schemas/events/kernel__v1__outbound.websocket.frame.schema.json`](./schemas/events/kernel__v1__outbound.websocket.frame.schema.json) | kernel | Outbound WebSocket frame observed | implemented |
| `kernel/v1/outbound.websocket.error` | [`./schemas/events/kernel__v1__outbound.websocket.error.schema.json`](./schemas/events/kernel__v1__outbound.websocket.error.schema.json) | kernel | Outbound WebSocket error | implemented |
| `kernel/v1/outbound.websocket.completed` | [`./schemas/events/kernel__v1__outbound.websocket.completed.schema.json`](./schemas/events/kernel__v1__outbound.websocket.completed.schema.json) | kernel | Outbound WebSocket completed/closed | implemented |
