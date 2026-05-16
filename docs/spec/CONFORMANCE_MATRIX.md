# Conformance Matrix

The conformance suite is the executable guardian of the charter. It should prove both positive behavior and hostile rejection behavior.

## Current conformance coverage

| Area | Case | Status |
|---|---|---:|
| session | open content-free session | implemented |
| events | authorized package appends own namespace event | implemented |
| events | package denied when writing without `events.append` | implemented in unit tests |
| events | package denied when reading without `events.read` | implemented |
| events | package denied when writing another namespace | implemented in unit tests |
| events | package denied when writing `kernel/...` | implemented in unit tests |
| events | closed session rejects append | implemented |
| package | valid manifest loads | implemented |
| package | host policy rejects disallowed entry | implemented in unit tests |
| package | unload removes registry record | implemented in unit tests |
| package | unload removes capability provider | implemented |
| capability | discover registered capability | implemented |
| capability | invoke rust_inproc echo through package trait | implemented |
| capability | ambiguous provider rejected | implemented in unit tests |
| official equality | official-looking package has no route precedence | implemented |
| hooks | veto fixture reports veto | implemented in unit tests |
| storage | SQLite persists/replays events | implemented in unit tests |
| protocol | method list contains no content methods | implemented in unit tests |
| schema | capability input schema rejects invalid input | implemented |
| schema | event payload schema rejects invalid payload | implemented |

## Required hostile conformance for Platform Host Alpha

| Area | Required case | Target phase |
|---|---|---|
| package execution | `rust_inproc` capability executes through package ABI, not hardcoded id logic | implemented |
| package execution | subprocess package completes JSON-RPC stdio handshake | Platform Host Alpha |
| package execution | subprocess timeout/crash/degraded behavior is enforced | Platform Host Alpha |
| package execution | package load goes through handshake/register/start states | Platform Host Alpha |
| capability | anonymous/dev caller behavior is explicitly marked host-only, not package privilege | Platform Host Alpha |
| capability | package caller without declared invoke permission is denied | Platform Host Alpha |
| capability | version mismatch fails | Platform Host Alpha |
| capability | duplicate providers produce ambiguous route unless policy selects one | implemented |
| capability | unloaded provider cannot be invoked | implemented |
| events | package without `events.read` cannot list events | implemented |
| events | closed session rejects append | implemented |
| events | sequence-range replay works | Platform Host Alpha |
| protocol | HTTP `/rpc` and in-process runtime share authorization behavior | Platform Host Alpha |
| protocol | host JSON-RPC stdio transport passes core conformance | Platform Host Alpha |
| hooks | hook ordering is stable | Platform Host Alpha |
| hooks | unload removes hook subscribers | Platform Host Alpha |
| hooks | before/after lifecycle hooks are dispatched by kernel operations | Platform Host Alpha |
| schema | manifest schema refs are resolvable | future |
| schema | capability input schema rejects invalid input | implemented |
| schema | capability output schema rejects invalid output | implemented in runtime path |
| schema | event payload schema rejects invalid payload when schema is declared | implemented |
| official equality | an `official/...` package has no special routing or permissions | implemented |
| official equality | kernel starts and conformance passes with no official packages loaded | implemented |

## CLI target output

`cargo run -p ygg-cli -- conformance` should evolve from a smoke test into a named case runner:

```text
session.open_empty                         PASS
event.append_authorized                    PASS
event.append_without_permission_denied     PASS
event.kernel_namespace_denied              PASS
event.read_without_permission_denied       PASS
event.closed_session_rejects_append        PASS
package.load_valid_manifest                PASS
package.unload_removes_capabilities        PASS
capability.invoke_rust_inproc              PASS
capability.ambiguous_provider_denied       PASS
official.no_privilege                      PASS
schema.capability_input_rejects_invalid    PASS
schema.event_payload_rejects_invalid       PASS
```

The suite should fail closed: any case listed as required for Platform Host Alpha must pass before that milestone can be declared complete.
