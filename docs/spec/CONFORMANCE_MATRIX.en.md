# Conformance Matrix

> [English](./CONFORMANCE_MATRIX.en.md) · [中文](./CONFORMANCE_MATRIX.md)

The conformance suite is the executable guardian of the charter. It proves both positive behavior and hostile rejection behavior. The current foundation is Platform Foundation Alpha + Play/Forge Surface Contract Beta. New cases land here as they are added; cases marked partial or future remain on the radar for Foundation Alpha Consolidation and Substrate hardening (see `docs/roadmap/NEXT_STEPS.md`).

## Current release-gate command

```bash
cargo test --workspace
cargo run -p ygg-cli -- conformance
```

Current matrix coverage: 165 implemented rows, backed by 180 named CLI conformance cases plus crate/service unit tests.

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
| substrate | permission grant survives SQLite-backed runtime rehydrate | implemented |
| secret refs | `secret_ref:`, `secretRef:`, `secret-ref:`, `host:` reference pattern validation | implemented |
| secret refs | raw secret in proposal payload is rejected | implemented |
| secret refs | raw secret in asset metadata is rejected | implemented |
| secret refs | official package has no secret-scanning bypass | implemented |
| env resolver | `EnvSecretResolver` allows resolution when env name is in allowlist (`secret_ref:env`, `secretRef:env`, `secret-ref:env`, `host:env`) | implemented |
| env resolver | `EnvSecretResolver` denies resolution when env name is not in allowlist; non-env vault and `host:<key>` rejected | implemented |
| env resolver | `EnvSecretResolver` missing env var returns typed error without leaking raw value | implemented |
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
| official packages | composition-lab exposes launch-plan, surface-graph, and compat-report capabilities with v2 descriptor diagnostics without privilege | implemented |
| official packages | asset-lab previews assets and drafts approval-gated import plans without privilege | implemented |
| official packages | projection-lab drafts rebuild plans and explains source events without privilege | implemented |
| official packages | playable-seed exposes reference entry/play/Forge/assistant surfaces and approval-gated edits | implemented |
| official packages | persona-lab imports and renders persona profiles with provenance without kernel ontology | implemented |
| official packages | knowledge-lab normalizes collections, matches entries, and returns plan-only injection output | implemented |
| official packages | context-lab assembles generic blocks with budget omissions and template rendering | implemented |
| official packages | text-transform-lab previews deterministic text transforms with trace and validation diagnostics | implemented |
| official packages | model-connector-lab validates profiles, rejects raw secrets, and returns no-network discovery plans | implemented |
| official packages | model-provider-lab as cloud API adapter lab lists eight cloud provider families, validates profiles rejecting raw secrets, package-local normalize_request covers eight dialects/endpoints, explains errors (401/429/529), outputs network_performed:false/inference_performed:false, no raw secret echoed; it is not the platform model abstraction | implemented |
| official packages | model-provider-lab cloud adapter invoke all eight provider families (OpenAI chat/responses, Anthropic messages, Gemini generateContent, OpenAI-compatible chat, OpenRouter chat/responses, DeepSeek chat, xAI chat/responses, Fireworks chat/responses; fake/local, auditable outbound_request_shape, raw credential rejected, openai_compatible missing/http base_url rejected, unsupported family diagnostic, executor_kind fake_local, live_call_supported false) | implemented |
| official packages | model-provider-lab cloud adapter normalize_stream eight families stream normalization (delta SSE, semantic SSE, typed chunk stream → StreamFrameEnvelope frames: start/chunk/progress/end/error/cancelled/timeout; terminal_frame_consistent; provider event input normalization; no raw secret echo; unsupported family empty frames + terminal_frame_consistent false) | implemented |
| outbound | model provider outbound shape fake executor (three-provider host/method/path/secret_ref shapes pass outbound boundary, call_count=3, executor_kind Fake) | implemented |
| official packages | model-routing-lab resolves deterministic route plans with explicit fallbacks and normalized params | implemented |
| official packages | pi-agent-runtime-lab produces no-inference/no-network run plans, approval-gated proposals, trace summaries, and discoverable surfaces | implemented |
| official packages | capability-tool-bridge-lab marks ambiguous provider rejected, explicit third-party provider available, official not preferred, missing provider rejected, denied preview reports missing permission, raw secret unsafe_blocked | implemented |
| official packages | inference-local-lab describe_capabilities: no network/secret required, transports include in_memory/local_process, operation_kinds include generate/classify/transform | implemented |
| official packages | inference-local-lab invoke non-HTTP succeeds with no URL/header/status/messages fields, network_performed=false, transport_performed=in_memory_fake | implemented |
| official packages | inference-local-lab invoke rejects http transport, HTTP-shaped fields (url/header/status_code), messages-shaped fields (messages/system/user/assistant), raw secret | implemented |
| official packages | inference-local-lab stream emits deterministic start/chunk/progress/end frames, no URL/header/status/provider_schema | implemented |
| official packages | inference-local-lab explain_error covers local/resource error classes (local_process_failed/local_resource_exhausted/local_model_not_loaded/local_inference_error/timeout/cancelled) | implemented |
| official packages | inference-playtest-lab draft_proposal produces proposal_draft with requires_user_approval=true, asset.put, source_inference provenance, no raw secret, not a chat message | implemented |
| official packages | inference-playtest-lab inspect_proposal returns risk/operations/permissions/provenance summary without applying | implemented |
| official packages | inference-playtest-lab rejected proposal cannot apply | implemented |
| official packages | inference-playtest-lab approve/apply succeeds, asset written, branch_plan + fork creates branch with proposal/source inference provenance | implemented |
| official packages | inference-playtest-lab output contains no messages/prompt/chat/kernel.model terms | implemented |
| in-process packages | non-official `/preview` suffix does not receive official asset-lab fallback behavior | implemented |
| in-process packages | unknown registered in-process capability fails loudly instead of returning generic fallback success | implemented |
| official packages | assistant-lab returns approval-gated proposals through grants | implemented |
| play-creation | blank loop exercises assistant proposal, branch, asset, projection | implemented |
| proposals | approved proposals can apply generic asset/projection operations | implemented |
| proposals | rejected or unapproved proposals cannot apply | implemented |
| package authoring | generated Python subprocess package passes local conformance | implemented |
| package authoring | generated TypeScript subprocess package passes local conformance | implemented |
| package authoring | generated experience package surfaces pass local conformance | implemented |
| composition | local composition descriptor validates package-provided surfaces | implemented |
| composition | composition descriptor v2: required capabilities pass, optional missing warns, required missing fails | implemented |
| official packages | composition-lab v2 diagnostics return surface/capability/permission/replacement fields and compat-report | implemented |
| replacement | third-party playable-seed surfaces discoverable through kernel.surface.contribution.list | implemented |
| replacement | third-party playable-seed capability invocation works through normal routing | implemented |
| replacement | ambiguous official+thirdparty equivalent capability rejects route without official priority | implemented |
| replacement | composition descriptor passes with third-party playable-seed replacement | implemented |
| replacement | third-party agent-runtime surfaces (assistant_action/forge_panel/home_card) discoverable through kernel.surface.contribution.list | implemented |
| replacement | third-party agent-runtime capability invocation produces no-inference/no-network, approval-gated proposal, provenance match | implemented |
| replacement | composition descriptor passes with third-party agent-runtime replacement, official is replacement_candidate only | implemented |
| network | package without network permission denied outbound, produces outbound.denied audit | implemented |
| network | allowlisted host+method allowed, produces redacted outbound.request audit | implemented |
| network | host/method mismatch denied | implemented |
| network | official package has no network bypass | implemented |
| network | audit records contain no raw secrets/bodies, only secret_ref and redaction_state | implemented |
| network | check_network_policy pure function tests | implemented |
| outbound | no permission executor not called — denied request never reaches executor | implemented |
| outbound | policy/audit request and executor request package/capability/host/method/secret_refs mismatch fails closed and never calls executor | implemented |
| outbound | allowlisted fake executor returns network_performed:false, executor_kind:fake, redacted audit | implemented |
| outbound | raw body_shape not persisted in audit; audit redaction_state redacted/not_captured | implemented |
| outbound | secret_refs stored as references only; raw secrets rejected/not echoed | implemented |
| outbound | host mismatch redirect denied; redirect_target check deferred to M4 | implemented |
| stream | normal lifecycle emits ordered frames/events | implemented |
| stream | cancel marks invocation cancelled and blocks further chunks | implemented |
| stream | timeout marks invocation timeout and blocks further chunks | implemented |
| stream | error terminal frame works | implemented |
| stream | non-streaming capability (streaming=false) rejected from stream | implemented |
| stream | no model/agent methods added to protocol | implemented |
| stream | capability.stream and capability.cancel dispatchable through protocol | implemented |
| package authoring | generated networked template passes check/conformance with network declarations, no raw secrets | implemented |
| package authoring | generated streaming template passes check/conformance with streaming capability | implemented |
| no-network readiness | faux-model-readiness package declares network permissions, provides streaming capability, uses secret_ref, no raw secrets | implemented |
| no-network readiness | faux-agent-readiness package has no network permissions, provides streaming capability, uses proposal/trace patterns, no raw secrets | implemented |
| outbound | live HTTP executor disabled by default; RuntimeConfig::default remains DenyAll | implemented |
| outbound | live HTTP executor rejects non-HTTPS URLs; no network attempted | implemented |
| outbound | live HTTP executor response shape contains no raw body/header/secret | implemented |
| outbound | kernel.outbound.execute public protocol: package principal determined from context (no spoofing), FakeOutboundExecutor + allowed network declaration succeeds with audit | implemented |
| outbound | kernel.outbound.execute spoofed package_id rejected, cannot act as another package | implemented |
| outbound | kernel.outbound.execute no network permission denied, executor not called | implemented |
| outbound | kernel.outbound.execute response contains no raw secret (secret_refs as references only) | implemented |
| outbound | kernel.outbound.execute `secret_headers` params parsed correctly, raw secret never in response | implemented |
| outbound | local loopback HTTP server secret injection: Authorization header actually arrives at server, raw secret not in protocol response/audit/log | implemented |
| outbound | DeepSeek SSE stream normalize canary: delta_sse start→chunk→end lifecycle, terminal_frame_consistent, no raw secrets | implemented |
| outbound | opt-in live DeepSeek conformance: default skip, only when YGG_LIVE_MODEL_TESTS=1 + DEEPSEEK_API_KEY | implemented |
| outbound | canary DeepSeek profile shape: normalize_request endpoint/dialect/stream_family correct, secret_ref placeholder no raw key | implemented |
| outbound | OpenAI Chat Completions loopback: Authorization Bearer arrives at server, POST /v1/chat/completions, body shape model+messages, raw secret not in response/audit | implemented |
| outbound | OpenAI Responses loopback: Authorization Bearer arrives, POST /v1/responses, body shape uses input field, raw secret not in response/audit | implemented |
| outbound | Anthropic Messages loopback: x-api-key secret header + anthropic-version static header arrive at server, POST /v1/messages, body shape content blocks, raw secret not in response/audit | implemented |
| outbound | Gemini generateContent loopback: x-goog-api-key secret header arrives at server, POST /v1beta/models/{model}:generateContent, body shape contents/parts, raw secret not in response/audit | implemented |
| outbound | missing secret fails closed: unavailable secret_ref produces error, no outbound request sent, no raw secret in error | implemented |
| outbound | provider normalize_request alignment: OpenAI chat+responses, Anthropic messages, Gemini generateContent endpoints/dialects match outbound.execute params, credential placeholders not raw | implemented |
| outbound | no raw secret leak across all providers: OpenAI/Anthropic/Gemini shapes through FakeOutboundExecutor, response+audit contain no raw secrets | implemented |
| outbound | static_headers safe allowlist: anthropic-version accepted, safe non-secret headers injected | implemented |
| outbound | static_headers block secrets: Authorization/x-api-key/Cookie in static_headers rejected, must use secret_headers | implemented |
| outbound | OpenRouter loopback headers: Authorization Bearer + HTTP-Referer + X-Title static headers arrive at server, POST /api/v1/chat/completions, raw secret not in response/audit | implemented |
| outbound | xAI loopback: Authorization Bearer arrives at server, POST /v1/chat/completions, reasoning/usage sanitized, raw secret not in response/audit | implemented |
| outbound | Fireworks loopback: Authorization Bearer arrives at server, POST /inference/v1/chat/completions, perf/usage metadata sanitized, raw secret not in response/audit | implemented |
| stream | DeepSeek reasoning stream normalization: reasoning_content → reasoning_delta frames, cache usage → progress frames, terminal_frame_consistent, no raw secrets | implemented |
| stream | OpenRouter mid-stream error normalization: error object after HTTP 200 → error frame with mid_stream_error provider_event | implemented |
| outbound | provider quirks sanitized fixtures: integrations/model-providers/fixtures/*.json contain no real keys or provider-looking raw keys, scan finds nothing | implemented |
| outbound | static_headers OpenRouter safe: http-referer/x-title on allowlist, not secret-bearing; Authorization/x-api-key still blocked | implemented |

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
composition.check_descriptor_v2             PASS
official.composition_lab                    PASS
official.composition_lab_diagnostics         PASS
official.asset_lab                         PASS
official.projection_lab                    PASS
official.playable_seed                     PASS
official.persona_lab                       PASS
official.knowledge_lab                     PASS
official.context_lab                       PASS
official.text_transform_lab                PASS
official.model_connector_lab               PASS
official.model_provider_lab                 PASS
official.model_provider_lab_invoke_core       PASS
official.model_provider_lab_normalize_stream  PASS
official.model_routing_lab                 PASS
official.pi_agent_runtime_lab              PASS
official.capability_tool_bridge_lab         PASS
official.inference_local_lab_describe_capabilities PASS
official.inference_local_lab_invoke          PASS
official.inference_local_lab_invoke_rejects_http PASS
official.inference_local_lab_stream          PASS
official.inference_local_lab_explain_error   PASS
official.inference_playtest_lab_draft         PASS
official.inference_playtest_lab_inspect       PASS
official.inference_playtest_lab_reject_apply_denied PASS
official.inference_playtest_lab_apply_and_branch PASS
official.inference_playtest_lab_no_chat_kernel_terms PASS
inproc.non_official_preview_rejected       PASS
inproc.unknown_capability_errors           PASS
replacement.thirdparty_seed_surfaces         PASS
replacement.thirdparty_seed_invocation       PASS
replacement.ambiguous_no_official_priority   PASS
replacement.composition_thirdparty           PASS
replacement.thirdparty_agent_runtime_surfaces   PASS
replacement.thirdparty_agent_runtime_invocation PASS
replacement.composition_agent_runtime_replacement PASS
substrate.permission_grant_rehydrate          PASS
secret.ref_validation                        PASS
secret.raw_blocked_in_proposal               PASS
secret.raw_blocked_in_asset_metadata         PASS
official.no_secret_bypass                    PASS
secret.env_resolver_allowed                  PASS
secret.env_resolver_denied                   PASS
secret.env_resolver_missing_no_leak          PASS
network.no_permission_denied                  PASS
network.allowlisted_host_method_allowed       PASS
network.host_method_mismatch_denied           PASS
network.official_no_network_bypass            PASS
network.audit_no_raw_secrets                  PASS
network.policy_pure_function                  PASS
outbound.no_permission_executor_not_called      PASS
outbound.allowlisted_fake_executor              PASS
outbound.raw_body_not_audited                   PASS
outbound.model_provider_shape_fake_executor   PASS
outbound.secret_refs_only                       PASS
outbound.host_mismatch_redirect_denied          PASS
stream.normal_lifecycle                       PASS
stream.cancel_blocks_chunks                   PASS
stream.timeout_blocks_chunks                  PASS
stream.error_terminal                         PASS
stream.non_streaming_rejected                 PASS
stream.no_model_agent_methods                 PASS
stream.protocol_dispatch                      PASS
package.generated_networked_template           PASS
package.generated_streaming_template           PASS
package.faux_model_readiness                   PASS
package.faux_agent_readiness                   PASS
outbound.live_http_default_disabled             PASS
outbound.live_http_rejects_insecure_url         PASS
outbound.live_http_redacted_shape               PASS
outbound.execute_package_allowed                 PASS
outbound.execute_spoofed_package_id_rejected     PASS
outbound.execute_no_permission_denied             PASS
outbound.execute_no_raw_secret_in_response        PASS
outbound.secret_headers_parsed                    PASS
outbound.live_loopback_secret_injection            PASS
stream.sse_normalize_deepseek_canary              PASS
outbound.live_deepseek_opt_in                     PASS
canary.deepseek_profile_shape                     PASS
outbound.openai_chat_loopback                     PASS
outbound.openai_responses_loopback                 PASS
outbound.anthropic_messages_loopback               PASS
outbound.gemini_generate_content_loopback          PASS
outbound.missing_secret_fails_closed               PASS
outbound.provider_normalize_request_alignment      PASS
outbound.no_raw_secret_leak_all_providers          PASS
outbound.static_headers_safe_allowlist             PASS
outbound.static_headers_block_secrets              PASS
outbound.openrouter_loopback_headers               PASS
outbound.xai_loopback                              PASS
outbound.fireworks_loopback                        PASS
stream.deepseek_reasoning_stream                   PASS
stream.openrouter_midstream_error                   PASS
outbound.provider_quirk_fixtures_no_secrets        PASS
outbound.static_headers_openrouter_safe             PASS
agentic_forge.describe_contract                       PASS
agentic_forge.start_run_plan_graph_working_state      PASS
agentic_forge.inspect_cancel_summarize                PASS
agentic_forge.raw_secret_blocked                      PASS
agentic_forge.no_kernel_agent_namespace                PASS
agentic_forge.create_candidate_branch_aware            PASS
agentic_forge.compare_candidate_stale_detection        PASS
agentic_forge.draft_promote_proposal_no_mutation       PASS
agentic_forge.stale_promote_blocked                    PASS
agentic_forge.archive_candidate_target_unchanged       PASS
agentic_forge.inference_node_deterministic_candidate_seed PASS
agentic_forge.replay_match_mismatch_flagged             PASS
agentic_forge.inference_output_privilege_escalation_rejected PASS
agentic_forge.cloud_adapter_needs_host_policy_no_network PASS
agentic_forge.inference_failure_taxonomy_recovery_hints PASS
agentic_forge.explain_tool_call_scoped_no_ambient_authority PASS
agentic_forge.record_observation_untrusted_large_output_redaction PASS
agentic_forge.tool_risk_injection_exfiltration_outbound    PASS
agentic_forge.replay_tool_plan_mismatch_flagged             PASS
agentic_forge.plan_toolchain_requires_explicit_provider_nested_delegation_blocked PASS
agentic_forge.thirdparty_replacement_shape_no_official_priority PASS
agentic_forge.no_official_priority_ordinary_package PASS
agentic_forge.hostile_injection_secret_blocked_cross_package PASS
agentic_forge.budget_deadline_contract_cancellation_consistent PASS
agentic_forge.cross_package_replay_mismatch_flagged PASS
```

The suite should fail closed: any case listed as required for Platform Host Alpha must pass before that milestone can be declared complete.
