# Conformance Matrix

The conformance suite is the executable guardian of the charter. It should prove both positive behavior and hostile rejection behavior.

## Current conformance coverage

| Area | Case | Status |
|---|---|---:|
| session | open content-free session | implemented |
| events | authorized package appends own namespace event | implemented |
| events | package denied when writing without `events.append` | implemented in unit tests |
| events | package denied when writing another namespace | implemented in unit tests |
| events | package denied when writing `kernel/...` | implemented in unit tests |
| package | valid manifest loads | implemented |
| package | host policy rejects disallowed entry | implemented in unit tests |
| package | unload removes registry record | implemented in unit tests |
| capability | discover registered capability | implemented |
| capability | invoke rust_inproc echo through package trait | implemented |
| capability | ambiguous provider rejected | implemented in unit tests |
| hooks | veto fixture reports veto | implemented in unit tests |
| storage | SQLite persists/replays events | implemented in unit tests |
| protocol | method list contains no content methods | implemented in unit tests |

## Required hostile conformance before official packages

| Area | Required case | Target phase |
|---|---|---|
| package execution | `rust_inproc` capability executes through package ABI, not hardcoded id logic | implemented |
| package execution | package load goes through handshake/register/start states | B |
| capability | anonymous/dev caller behavior is explicitly marked host-only, not package privilege | C |
| capability | package caller without declared invoke permission is denied | C |
| capability | version mismatch fails | C |
| capability | duplicate providers produce ambiguous route unless policy selects one | C |
| capability | unloaded provider cannot be invoked | C |
| events | package without `events.read` cannot list events | C |
| events | closed session rejects append | C |
| events | sequence-range replay works | C |
| hooks | hook ordering is stable | C |
| hooks | unload removes hook subscribers | C |
| hooks | before/after lifecycle hooks are dispatched by kernel operations | C |
| schema | manifest schema refs are resolvable | D |
| schema | capability input schema rejects invalid input | D |
| schema | capability output schema rejects invalid output | D |
| schema | event payload schema rejects invalid payload when schema is declared | D |
| official equality | an `official/...` package has no special routing or permissions | C |
| official equality | kernel starts and conformance passes with no official packages loaded | C |

## CLI target output

`cargo run -p ygg-cli -- conformance` should evolve from a smoke test into a named case runner:

```text
session.open_empty                         PASS
event.append_authorized                    PASS
event.append_without_permission_denied     PASS
event.kernel_namespace_denied              PASS
package.load_valid_manifest                PASS
package.unload_removes_capabilities        PASS
capability.invoke_rust_inproc              PASS
capability.ambiguous_provider_denied       PASS
official.no_privilege                      PASS
```

The suite should fail closed: any unimplemented case listed as required for the current phase must fail CI.
