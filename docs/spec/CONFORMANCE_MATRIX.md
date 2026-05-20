# Conformance 矩阵

> [English](./CONFORMANCE_MATRIX.en.md) · [中文](./CONFORMANCE_MATRIX.md)

Conformance 套件是 charter 的可执行守卫。它同时证明正向行为和 hostile 拒绝行为。当前基础是 Platform Foundation Alpha + Play/Forge Surface Contract Beta。新用例在添加时收入此处；标记为 partial 或 future 的用例仍在 Foundation Alpha Consolidation 和底座 hardening 的雷达上（见 `docs/roadmap/NEXT_STEPS.md`）。

## 当前发布门槛命令

```bash
cargo test --workspace
cargo run -p ygg-cli -- conformance
```

当前矩阵覆盖：180 个 implemented rows，由 226 个具名 CLI conformance 用例 + crate/service 单元测试支撑。

## 当前 conformance 覆盖

| 领域 | 用例 | 状态 |
|---|---|---:|
| session | 开启内容无关 session | implemented |
| events | 已授权包追加自身 namespace 事件 | implemented |
| events | 包在无 `events.append` 时被拒绝写入 | implemented in unit tests |
| events | 包在无 `events.read` 时被拒绝读取 | implemented |
| events | 包被拒绝写入他人 namespace | implemented in unit tests |
| events | 包被拒绝写入 `kernel/...` | implemented in unit tests |
| events | 已关闭 session 拒绝追加 | implemented |
| events | 带过滤条件的 sequence-range replay | implemented |
| package | 有效 manifest 加载成功 | implemented |
| package | lifecycle 时间线发出 loading/starting/ready/loaded | implemented |
| package | 重启 subprocess 包 | implemented |
| package | 捕获 subprocess stderr 日志 | implemented |
| package | host 策略拒绝不允许的 entry | implemented in unit tests |
| package | unload 移除注册记录 | implemented in unit tests |
| package | unload 移除 capability provider | implemented |
| capability | 发现已注册的 capability | implemented |
| capability | 通过 package trait 调用 rust_inproc echo | implemented |
| capability | 模糊 provider 被拒绝 | implemented in unit tests |
| capability | 显式 provider 选择解决重复 provider | implemented |
| capability | 版本约束过滤 provider | implemented |
| official equality | 官方外观的包无路由优先 | implemented |
| hooks | veto fixture 报告 veto | implemented in unit tests |
| hooks | 按 precedence/package/handler 稳定排序 | implemented |
| hooks | before event append veto 阻止操作 | implemented |
| hooks | before event append metadata 变更生效 | implemented |
| hooks | 包拥有的 hook handler capability 被调用 | implemented |
| hooks | unload 移除 hook 订阅 | implemented |
| storage | SQLite 持久化/replay 事件 | implemented in unit tests |
| assets | put/get/list 不透明 asset | implemented |
| sessions | fork session 并列出 branch 族系 | implemented |
| projections | 注册并 rebuild 通用事件计数 projection | implemented |
| substrate | SQLite 事件日志 rehydrate asset、branch 和 projection | implemented |
| substrate | permission grant 在 SQLite-backed runtime rehydrate 后仍存在 | implemented |
| secret refs | `secret_ref:`、`secretRef:`、`secret-ref:`、`host:` reference pattern validation | implemented |
| secret refs | proposal payload 中的 raw secret 会被拒绝 | implemented |
| secret refs | asset metadata 中的 raw secret 会被拒绝 | implemented |
| secret refs | 官方包没有 secret-scanning bypass | implemented |
| env resolver | `EnvSecretResolver` 在 env name 于 allowlist 中时允许解析（`secret_ref:env`、`secretRef:env`、`secret-ref:env`、`host:env`） | implemented |
| env resolver | `EnvSecretResolver` 在 env name 不在 allowlist 中时拒绝解析；非 env vault 和 `host:<key>` 被拒绝 | implemented |
| env resolver | `EnvSecretResolver` 缺失 env var 返回 typed error，不泄漏 raw value | implemented |
| protocol | 方法列表不包含内容方法 | implemented in unit tests |
| protocol | 结构化权限错误码 | implemented |
| protocol | in-process 协议分发器调用 host.info | implemented |
| protocol | in-process 协议分发器调用 capability | implemented |
| protocol | HTTP `/rpc` 返回协议信封 | implemented in service tests |
| protocol | host stdio 响应协议信封 | implemented by CLI validation |
| principal | 包上下文覆盖调用者提供的 event writer | implemented |
| principal | 包上下文覆盖调用者提供的 capability caller | implemented |
| principal | human 和 assistant 协议 principal 存在 | implemented |
| permissions | grant/revoke/list/audit 协议 | implemented |
| permissions | assistant capability 调用需要显式授权 | implemented |
| schema | capability input schema 拒绝无效输入 | implemented |
| schema | event payload schema 拒绝无效 payload | implemented |
| subprocess | JSON-RPC stdio 包加载并报告 ready | implemented |
| subprocess | JSON-RPC stdio capability 调用正常工作 | implemented |
| subprocess | 错误握手被拒绝 | implemented |
| subprocess | 调用超时导致包降级 | implemented |
| subprocess | 无效 subprocess 输出 schema 被拒绝 | implemented |
| subprocess | unload 移除 subprocess capability | implemented |
| service | SSE 事件订阅端点 replay 和 tail 事件 | implemented |
| host | diagnostics 报告包/capability/hook | implemented |
| host | profile 自动加载配置的包 | implemented |
| surfaces | 包贡献的类型化 surface 描述符可以列出、描述和过滤 | implemented |
| official packages | 基础包无特权加载和调用 | implemented |
| official packages | composition-lab 以无特权方式暴露 launch-plan、surface-graph 与 compat-report capabilities，支持 v2 descriptor 诊断 | implemented |
| official packages | asset-lab 以无特权方式 preview assets 并生成需要审批的 import plans | implemented |
| official packages | projection-lab 以无特权方式生成 rebuild plans 并解释 source events | implemented |
| official packages | playable-seed 暴露 reference entry/play/Forge/assistant surfaces 以及需要审批的 edits | implemented |
| official packages | persona-lab 以无 kernel ontology 的方式 import 并 render persona profiles，且带 provenance | implemented |
| official packages | knowledge-lab normalize collections、match entries，并返回 plan-only injection output | implemented |
| official packages | context-lab 组装 generic blocks，包含 budget omissions 与 template rendering | implemented |
| official packages | text-transform-lab preview deterministic text transforms，包含 trace 与 validation diagnostics | implemented |
| official packages | model-connector-lab validate profiles、拒绝 raw secrets，并返回 no-network discovery plans | implemented |
| official packages | model-provider-lab 作为 cloud API adapter lab 列出八家 cloud provider families、validate profiles 拒绝 raw secret、package-local normalize_request 覆盖八家 dialects/endpoints、explain errors（401/429/529）、output 含 network_performed:false/inference_performed:false、无 raw secret echo；它不是平台模型抽象 | implemented |
| official packages | model-provider-lab cloud adapter invoke 全部八家 provider（OpenAI chat/responses、Anthropic messages、Gemini generateContent、OpenAI-compatible chat、OpenRouter chat/responses、DeepSeek chat、xAI chat/responses、Fireworks chat/responses；fake/local、outbound_request_shape 可审计、raw credential rejected、openai_compatible 缺 base_url 或 http base_url 拒绝、unsupported family diagnostic、executor_kind fake_local、live_call_supported false） | implemented |
| official packages | model-provider-lab cloud adapter normalize_stream 八家 provider stream normalization（delta SSE、semantic SSE、typed chunk stream → StreamFrameEnvelope frames：start/chunk/progress/end/error/cancelled/timeout；terminal_frame_consistent；provider event 输入归一化；raw secret 不 echo；unsupported family empty frames + terminal_frame_consistent false） | implemented |
| outbound | model provider outbound shape fake executor（三 provider host/method/path/secret_ref shape 通过 outbound boundary、call_count=3、executor_kind Fake） | implemented |
| official packages | model-routing-lab resolve deterministic route plans，包含 explicit fallbacks 与 normalized params | implemented |
| official packages | pi-agent-runtime-lab 生成 no-inference/no-network run plans、approval-gated proposals、trace summaries，且 surfaces 可发现 | implemented |
| official packages | capability-tool-bridge-lab 标记 ambiguous provider rejected、explicit third-party provider 可用、official 不优先、missing provider rejected、denied preview 报告 missing permission、raw secret unsafe_blocked | implemented |
| official packages | inference-local-lab describe_capabilities 不需要 network/secret，transports include in_memory/local_process，operation_kinds include generate/classify/transform | implemented |
| official packages | inference-local-lab invoke non-HTTP succeeds，无 URL/header/status/messages 字段，network_performed=false，transport_performed=in_memory_fake | implemented |
| official packages | inference-local-lab invoke rejects http transport、HTTP-shaped 字段（url/header/status_code）、messages-shaped 字段（messages/system/user/assistant）、raw secret | implemented |
| official packages | inference-local-lab stream emits deterministic start/chunk/progress/end frames，无 URL/header/status/provider_schema | implemented |
| official packages | inference-local-lab explain_error 覆盖 local/resource 错误类（local_process_failed/local_resource_exhausted/local_model_not_loaded/local_inference_error/timeout/cancelled） | implemented |
| official packages | inference-playtest-lab draft_proposal 产 proposal_draft，含 requires_user_approval=true、asset.put、source_inference provenance、无 raw secret、不是 chat message | implemented |
| official packages | inference-playtest-lab inspect_proposal 返回 risk/operations/permissions/provenance summary，不 apply | implemented |
| official packages | inference-playtest-lab 被拒绝的 proposal 不能 apply | implemented |
| official packages | inference-playtest-lab approve/apply 成功，asset 被写入，branch_plan + fork 创建 branch，branch metadata 包含 proposal/source inference provenance | implemented |
| official packages | inference-playtest-lab 输出不含 messages/prompt/chat/kernel.model 等术语 | implemented |
| in-process packages | non-official `/preview` suffix 不会获得 official asset-lab fallback 行为 | implemented |
| in-process packages | unknown registered in-process capability loud fail，而不是返回 generic fallback success | implemented |
| official packages | assistant-lab 通过授权返回需要审批的 proposal | implemented |
| play-creation | 空白循环演练 assistant proposal、branch、asset、projection | implemented |
| proposals | 已批准的 proposal 可以执行通用 asset/projection 操作 | implemented |
| proposals | 被拒绝或未批准的 proposal 不能执行 | implemented |
| package authoring | 生成的 Python subprocess 包通过本地 conformance | implemented |
| package authoring | 生成的 TypeScript subprocess 包通过本地 conformance | implemented |
| package authoring | 生成的 experience 包 surface 通过本地 conformance | implemented |
| composition | 本地 composition 描述符验证包提供的 surface | implemented |
| composition | composition 描述符 v2：required capabilities 通过、optional 缺失仅警告、required 缺失失败 | implemented |
| official packages | composition-lab v2 诊断返回 surface/capability/permission/replacement 字段与 compat-report | implemented |
| replacement | 第三方 playable-seed surface 通过 kernel.surface.contribution.list 可发现 | implemented |
| replacement | 第三方 playable-seed 能力调用通过正常路由工作 | implemented |
| replacement | 歧义的 official+thirdparty 等效能力拒绝路由，无官方优先 | implemented |
| replacement | composition 描述符通过第三方 playable-seed 替换 | implemented |
| replacement | 第三方 agent-runtime surfaces（assistant_action/forge_panel/home_card）通过 kernel.surface.contribution.list 可发现 | implemented |
| replacement | 第三方 agent-runtime 能力调用产生 no-inference/no-network、approval-gated proposal、provenance 匹配 | implemented |
| replacement | composition 描述符通过第三方 agent-runtime 替换，official 仅 replacement_candidate | implemented |
| network | 无 network permission 的包被拒绝出站，产生 outbound.denied 审计 | implemented |
| network | allowlisted host+method 允许，产生 redacted outbound.request 审计 | implemented |
| network | host/method 不匹配被拒绝 | implemented |
| network | 官方包无 network bypass | implemented |
| network | 审计记录不包含 raw secret/body，只包含 secret_ref 和 redaction_state | implemented |
| network | check_network_policy 纯函数测试 | implemented |
| outbound | 无权限时 executor 不被调用 — 被拒绝的请求不会到达 executor | implemented |
| outbound | policy/audit request 与 executor request 的 package/capability/host/method/secret_refs 不一致时 fail-closed，executor 不被调用 | implemented |
| outbound | allowlisted fake executor 返回 network_performed:false、executor_kind:fake、redacted audit | implemented |
| outbound | raw body_shape 不持久化到审计；审计 redaction_state 为 redacted/not_captured | implemented |
| outbound | secret_refs 仅存储为引用；raw secret 被拒绝/不回显 | implemented |
| outbound | host 不匹配时 redirect 被拒绝；redirect_target 检查延后 M4 | implemented |
| stream | 正常生命周期发出有序 frame/event | implemented |
| stream | cancel 标记 invocation 为 cancelled 并阻断后续 chunk | implemented |
| stream | timeout 标记 invocation 为 timeout 并阻断后续 chunk | implemented |
| stream | error 终端 frame 正常工作 | implemented |
| stream | 非 streaming 能力（streaming=false）被拒绝 | implemented |
| stream | 协议中无 model/agent 方法 | implemented |
| stream | capability.stream 和 capability.cancel 可通过协议分发 | implemented |
| package authoring | 生成的 networked 模板通过 check/conformance，含网络声明，无 raw secrets | implemented |
| package authoring | 生成的 streaming 模板通过 check/conformance，含 streaming capability | implemented |
| no-network readiness | faux-model-readiness 包声明网络权限、提供 streaming capability、使用 secret_ref、无 raw secrets | implemented |
| no-network readiness | faux-agent-readiness 包无网络权限、提供 streaming capability、使用 proposal/trace 模式、无 raw secrets | implemented |
| outbound | live HTTP executor 默认关闭；RuntimeConfig::default 仍 DenyAll | implemented |
| outbound | live HTTP executor 拒绝非 HTTPS URL；无网络尝试 | implemented |
| outbound | live HTTP executor response shape 不含 raw body/header/secret | implemented |
| outbound | kernel.outbound.execute 公开协议：package principal 通过 context 确定 package_id 不能 spoof，FakeOutboundExecutor + allowed network declaration 成功且 audit 产生 | implemented |
| outbound | kernel.outbound.execute spoofed package_id 被拒绝，不能代替其他 package | implemented |
| outbound | kernel.outbound.execute 无 network permission denied，executor 不调用 | implemented |
| outbound | kernel.outbound.execute response 不含 raw secret（secret_refs 仅引用） | implemented |
| outbound | kernel.outbound.execute `secret_headers` params 解析正确，raw secret 不出现在 response | implemented |
| outbound | local loopback HTTP server secret injection：Authorization header 真实到达 server，raw secret 不在 protocol response/audit/log | implemented |
| outbound | DeepSeek SSE stream normalize canary：delta_sse start→chunk→end lifecycle，terminal_frame_consistent，no raw secrets | implemented |
| outbound | opt-in live DeepSeek conformance：默认跳过，YGG_LIVE_MODEL_TESTS=1 + DEEPSEEK_API_KEY 时才尝试 | implemented |
| outbound | canary DeepSeek profile shape：normalize_request endpoint/dialect/stream_family 正确，secret_ref placeholder 不含 raw key | implemented |
| outbound | OpenAI Chat Completions loopback：Authorization Bearer 到达 server，POST /v1/chat/completions，body shape model+messages，raw secret 不在 response/audit | implemented |
| outbound | OpenAI Responses loopback：Authorization Bearer 到达 server，POST /v1/responses，body shape 使用 input 字段，raw secret 不在 response/audit | implemented |
| outbound | Anthropic Messages loopback：x-api-key secret header + anthropic-version static header 到达 server，POST /v1/messages，body shape content blocks，raw secret 不在 response/audit | implemented |
| outbound | Gemini generateContent loopback：x-goog-api-key secret header 到达 server，POST /v1beta/models/{model}:generateContent，body shape contents/parts，raw secret 不在 response/audit | implemented |
| outbound | missing secret fails closed：不可用的 secret_ref 产生错误，无 outbound 请求发出，错误中不含 raw secret | implemented |
| outbound | provider normalize_request alignment：OpenAI chat+responses、Anthropic messages、Gemini generateContent endpoint/dialect 匹配 outbound.execute 参数，credential placeholder 非 raw | implemented |
| outbound | no raw secret leak across all providers：OpenAI/Anthropic/Gemini shapes 通过 FakeOutboundExecutor，response+audit 不含 raw secrets | implemented |
| outbound | static_headers safe allowlist：anthropic-version 接受，安全非 secret headers 可注入 | implemented |
| outbound | static_headers block secrets：Authorization/x-api-key/Cookie 在 static_headers 中被拒绝，必须使用 secret_headers | implemented |
| outbound | OpenRouter loopback headers：Authorization Bearer + HTTP-Referer + X-Title static headers 到达 server，POST /api/v1/chat/completions，raw secret 不在 response/audit | implemented |
| outbound | xAI loopback：Authorization Bearer 到达 server，POST /v1/chat/completions，reasoning/usage sanitized，raw secret 不在 response/audit | implemented |
| outbound | Fireworks loopback：Authorization Bearer 到达 server，POST /inference/v1/chat/completions，perf/usage metadata sanitized，raw secret 不在 response/audit | implemented |
| stream | DeepSeek reasoning stream normalization：reasoning_content → reasoning_delta frames，cache usage → progress frames，terminal_frame_consistent，no raw secrets | implemented |
| stream | OpenRouter mid-stream error normalization：error object after HTTP 200 → error frame with mid_stream_error provider_event | implemented |
| outbound | provider quirks sanitized fixtures：integrations/model-providers/fixtures/*.json 不含真实 key 或 provider-looking raw key，scan 无 findings | implemented |
| outbound | static_headers OpenRouter safe：http-referer/x-title 在 allowlist 上，非 secret-bearing；Authorization/x-api-key 仍被阻止 | implemented |
| official packages | experience-observability-lab describe_observability 返回 8 项能力、3 个 surface、output shapes，无 forbidden namespace | implemented |
| official packages | experience-observability-lab summarize_session_health 从协议可见引用派生状态，不读 SQLite | implemented |
| official packages | experience-observability-lab summarize_package_health 从协议可见引用返回 package health | implemented |
| official packages | experience-observability-lab summarize_agent_run_health 从协议可见引用返回 agent run health | implemented |
| official packages | experience-observability-lab trace_proposal_causality 返回因果链，每步含 content_address | implemented |
| official packages | experience-observability-lab summarize_cost_latency 从 outbound audit 引用返回 cost/latency summary，无 raw secret | implemented |
| official packages | experience-observability-lab list_failure_breadcrumbs 从协议可见 event 引用返回 failure breadcrumbs | implemented |
| official packages | experience-observability-lab summarize_guardrails 从协议可见 audit 引用返回 guardrail/audit summary | implemented |
| official packages | experience-observability-lab 任何输出不含 kernel.observability.* / kernel.experience.* namespace | implemented |
| official packages | experience-observability-lab 所有能力输入阻断 raw secret | implemented |
| official packages | memory-lab describe_memory_contract 返回 9 项能力、3 个 surface、output shapes，无 forbidden namespace | implemented |
| official packages | memory-lab record_memory 产出 memory_record 含 content_address / branch_ref / knowledge_refs | implemented |
| official packages | memory-lab retrieve_memory 确定性关键词匹配，branch-aware 过滤，无 embedding/network | implemented |
| official packages | memory-lab trace_retrieval 产出确定性 retrieval trace | implemented |
| official packages | memory-lab draft_memory_update 仅产出 proposal/update draft，不直接改持久状态，requires_user_approval=true | implemented |
| official packages | memory-lab apply_memory_correction 产出 correction shape，proposal-gated | implemented |
| official packages | memory-lab draft_forget_redaction 产出 redaction plan，不直接删除 | implemented |
| official packages | memory-lab branch_memory_view 按 branch 过滤记忆记录 | implemented |
| official packages | memory-lab 任何输出不含 kernel.memory.* / kernel.experience.* namespace | implemented |
| official packages | memory-lab 所有能力输入阻断 raw secret | implemented |

## Platform Host Alpha 必需的 hostile conformance

| 领域 | 必需用例 | 目标阶段 |
|---|---|---|
| package execution | `rust_inproc` capability 通过 package ABI 执行，而非硬编码 id 逻辑 | implemented |
| package execution | subprocess 包完成 JSON-RPC stdio 握手 | Platform Host Alpha |
| package execution | subprocess 超时/崩溃/降级行为被强制执行 | Platform Host Alpha |
| package execution | 包加载经历 loading/starting/ready 状态 | implemented |
| capability | anonymous/dev 调用者行为被显式标记为 host-only，非包特权 | Platform Host Alpha |
| capability | 未声明 invoke 权限的包调用者被拒绝 | Platform Host Alpha |
| capability | 版本不匹配失败 | partial |
| capability | 重复 provider 在调用者未选择 provider 时产生 ambiguous route | implemented |
| capability | 已卸载的 provider 不能被调用 | implemented |
| events | 无 `events.read` 的包不能列出事件 | implemented |
| events | 已关闭 session 拒绝追加 | implemented |
| events | sequence-range replay 正常工作 | implemented |
| protocol | HTTP `/rpc` 和 in-process 运行时共享授权行为 | Platform Host Alpha |
| protocol | host JSON-RPC stdio 传输层通过核心 conformance | Platform Host Alpha |
| hooks | hook 排序稳定 | implemented |
| hooks | unload 移除 hook 订阅者 | implemented |
| hooks | before/after lifecycle hook 由内核操作分发 | partial |
| hooks | 包拥有的 hook handler capability 被调用 | implemented |
| schema | manifest schema 引用可解析 | future |
| schema | capability input schema 拒绝无效输入 | implemented |
| schema | capability 输出 schema 拒绝无效输出 | implemented in runtime path |
| schema | 声明了 schema 时 event payload schema 拒绝无效 payload | implemented |
| official equality | `official/...` 包没有特殊路由或权限 | implemented |
| official equality | 内核在未加载任何官方包时启动且 conformance 通过 | implemented |

## CLI 目标输出

`cargo run -p ygg-cli -- conformance` 应从一个冒烟测试演进为具名用例运行器：

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
official.composition_lab                   PASS
official.composition_lab_diagnostics       PASS
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
substrate.permission_grant_rehydrate         PASS
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
playable_board.describe_contract_shape PASS
playable_board.launch_and_player_actions PASS
playable_board.checkpoint_recovery_shape PASS
playable_board.request_change_no_chat PASS
playable_board.bind_agent_run_scoped PASS
playable_board.candidate_proposal_no_target_mutation PASS
playable_board.reject_approve_fork_proof PASS
playable_board.thirdparty_no_official_priority PASS
playable_board.no_forbidden_namespace PASS
playable_board.no_raw_secrets PASS
playable_board.content_address_stable PASS
playable_board.checkpoint_metadata PASS
playable_board.provenance_graph PASS
playable_board.state_diff_preview PASS
playable_board.describe_asset_provenance PASS
playable_board.beta2_no_raw_secrets PASS
official.asset_lab_content_address PASS
official.asset_lab_provenance_graph PASS
official.projection_lab_state_snapshot PASS
experience_observability.contract_shape PASS
experience_observability.session_health PASS
experience_observability.package_health PASS
experience_observability.agent_run_health PASS
experience_observability.proposal_causality PASS
experience_observability.cost_latency_summary PASS
experience_observability.failure_breadcrumbs PASS
experience_observability.guardrail_audit_summary PASS
experience_observability.no_forbidden_namespace PASS
experience_observability.no_raw_secrets PASS
memory_lab.contract_shape PASS
memory_lab.record_memory PASS
memory_lab.retrieve_memory PASS
memory_lab.trace_retrieval PASS
memory_lab.draft_update_proposal_only PASS
memory_lab.correction_proposal_gated PASS
memory_lab.forget_redaction_plan PASS
memory_lab.branch_view PASS
memory_lab.no_forbidden_namespace PASS
memory_lab.no_raw_secrets PASS
```

该套件应该以封闭失败为原则：任何列为 Platform Host Alpha 必需的用例必须通过，该里程碑才能被宣布完成。
