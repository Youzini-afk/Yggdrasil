# Conformance Matrix

The conformance suite is the executable guardian of the charter. It proves both positive behavior and hostile rejection behavior. Platform Host Alpha is not complete until every case marked required for the milestone is either implemented or deliberately reclassified as deferred with a documented reason.

## Current release-gate command

```bash
cargo test --workspace
cargo run -p ygg-cli -- conformance
```

Current named conformance coverage: 51 CLI cases plus crate/service unit tests.

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
| events | sequence-range replay with filters | implemented |
| package | valid manifest loads | implemented |
| package | lifecycle timeline emits loading/starting/ready/loaded | implemented |
| package | restart subprocess package | implemented |
| package | capture subprocess stderr logs | implemented |
| package | host policy rejects disallowed entry | implemented in unit tests |
| package | unload removes registry record | implemented in unit tests |
| package | unload removes capability provider | implemented |
| capability | discover registered capability | implemented |
| capability | invoke rust_inproc echo through package trait | implemented |
| capability | ambiguous provider rejected | implemented in unit tests |
| capability | explicit provider selection resolves duplicate providers | implemented |
| capability | version constraint filters providers | implemented |
| official equality | official-looking package has no route precedence | implemented |
| hooks | veto fixture reports veto | implemented in unit tests |
| hooks | stable ordering by precedence/package/handler | implemented |
| hooks | before event append veto blocks operation | implemented |
| hooks | before event append metadata mutation is applied | implemented |
| hooks | package-owned hook handler capability is invoked | implemented |
| hooks | unload removes hook subscription | implemented |
| storage | SQLite persists/replays events | implemented in unit tests |
| assets | put/get/list opaque assets | implemented |
| sessions | fork session and list branch lineage | implemented |
| projections | register and rebuild generic event-count projection | implemented |
| substrate | SQLite event log rehydrates assets, branches, and projections | implemented |
| protocol | method list contains no content methods | implemented in unit tests |
| protocol | structured permission error code | implemented |
| protocol | in-process protocol dispatcher calls host.info | implemented |
| protocol | in-process protocol dispatcher invokes capability | implemented |
| protocol | HTTP `/rpc` returns protocol envelope | implemented in service tests |
| protocol | host stdio responds to protocol envelope | implemented by CLI validation |
| principal | package context overrides caller-supplied event writer | implemented |
| principal | package context overrides caller-supplied capability caller | implemented |
| principal | human and assistant protocol principals exist | implemented |
| permissions | grant/revoke/list/audit protocol | implemented |
| permissions | assistant capability invoke requires explicit grant | implemented |
| schema | capability input schema rejects invalid input | implemented |
| schema | event payload schema rejects invalid payload | implemented |
| subprocess | JSON-RPC stdio package loads and reports ready | implemented |
| subprocess | JSON-RPC stdio capability invoke works | implemented |
| subprocess | bad handshake is rejected | implemented |
| subprocess | invoke timeout degrades package | implemented |
| subprocess | invalid subprocess output schema is rejected | implemented |
| subprocess | unload removes subprocess capability | implemented |
| service | SSE event subscribe endpoint replays and tails events | implemented |
| host | diagnostics reports packages/capabilities/hooks | implemented |
| host | profile autoload loads configured packages | implemented |
| surfaces | package-contributed typed surface descriptors can be listed, described, and filtered | implemented |
| official packages | foundation packages load and invoke without privilege | implemented |
| official packages | assistant-lab returns approval-gated proposals through grants | implemented |
| play-creation | blank loop exercises assistant proposal, branch, asset, projection | implemented |
| proposals | approved proposals can apply generic asset/projection operations | implemented |
| proposals | rejected or unapproved proposals cannot apply | implemented |
| package authoring | generated Python subprocess package passes local conformance | implemented |
| package authoring | generated TypeScript subprocess package passes local conformance | implemented |
| package authoring | generated experience package surfaces pass local conformance | implemented |
| composition | local composition descriptor validates package-provided surfaces | implemented |

## Required hostile conformance for Platform Host Alpha

| Area | Required case | Target phase |
|---|---|---|
| package execution | `rust_inproc` capability executes through package ABI, not hardcoded id logic | implemented |
| package execution | subprocess package completes JSON-RPC stdio handshake | Platform Host Alpha |
| package execution | subprocess timeout/crash/degraded behavior is enforced | Platform Host Alpha |
| package execution | package load goes through loading/starting/ready states | implemented |
| capability | anonymous/dev caller behavior is explicitly marked host-only, not package privilege | Platform Host Alpha |
| capability | package caller without declared invoke permission is denied | Platform Host Alpha |
| capability | version mismatch fails | partial |
| capability | duplicate providers produce ambiguous route unless caller selects provider | implemented |
| capability | unloaded provider cannot be invoked | implemented |
| events | package without `events.read` cannot list events | implemented |
| events | closed session rejects append | implemented |
| events | sequence-range replay works | implemented |
| protocol | HTTP `/rpc` and in-process runtime share authorization behavior | Platform Host Alpha |
| protocol | host JSON-RPC stdio transport passes core conformance | Platform Host Alpha |
| hooks | hook ordering is stable | implemented |
| hooks | unload removes hook subscribers | implemented |
| hooks | before/after lifecycle hooks are dispatched by kernel operations | partial |
| hooks | package-owned hook handler capability is invoked | implemented |
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
event.range_replay                         PASS
package.load_valid_manifest                PASS
package.unload_removes_capabilities        PASS
capability.invoke_rust_inproc              PASS
capability.ambiguous_provider_denied       PASS
capability.explicit_provider_selected      PASS
official.no_privilege                      PASS
schema.capability_input_rejects_invalid    PASS
schema.event_payload_rejects_invalid       PASS
protocol.structured_permission_error       PASS
permission.grant_revoke_audit              PASS
permission.assistant_capability_grant      PASS
protocol.call_host_info                    PASS
protocol.call_capability_in_process        PASS
principal.package_cannot_self_assert_writer PASS
principal.package_cannot_self_assert_capability_caller PASS
subprocess.load_ready                      PASS
subprocess.invoke_echo                     PASS
package.lifecycle_timeline                 PASS
package.logs_capture                       PASS
package.restart_subprocess                 PASS
host.diagnostics                           PASS
host.profile_autoload                      PASS
surface.contribution_list                  PASS
official.foundation_packages               PASS
official.assistant_lab_proposal            PASS
play_creation.blank_loop                   PASS
proposal.lifecycle_apply                   PASS
proposal.reject_and_apply_denied           PASS
asset.put_get_list                         PASS
session.fork_branch                        PASS
projection.rebuild                         PASS
substrate.sqlite_rehydrate                 PASS
subprocess.bad_handshake                   PASS
subprocess.invoke_timeout                  PASS
subprocess.invalid_output_schema           PASS
subprocess.unload_removes_capability       PASS
hook.ordering_stable                       PASS
hook.veto_blocks_event_append              PASS
hook.metadata_mutation_allowed             PASS
hook.package_owned_handler                 PASS
hook.unload_removes_subscription           PASS
package.generated_subprocess_conformance   PASS
package.generated_typescript_subprocess_conformance PASS
package.generated_experience_template      PASS
composition.check_descriptor               PASS
```

The suite should fail closed: any case listed as required for Platform Host Alpha must pass before that milestone can be declared complete.
