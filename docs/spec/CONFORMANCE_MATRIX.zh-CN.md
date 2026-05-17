# Conformance 矩阵

> [English](./CONFORMANCE_MATRIX.md) · [中文](./CONFORMANCE_MATRIX.zh-CN.md)

Conformance 套件是 charter 的可执行守卫。它同时证明正向行为和 hostile 拒绝行为。当前基础是 Platform Foundation Alpha + Play/Forge Surface Contract Beta。新用例在添加时收入此处；标记为 partial 或 future 的用例仍在 Foundation Alpha Consolidation 和底座 hardening 的雷达上（见 `docs/roadmap/NEXT_STEPS.md`）。

## 当前发布门槛命令

```bash
cargo test --workspace
cargo run -p ygg-cli -- conformance
```

当前具名 conformance 覆盖：59 个 CLI 用例 + crate/service 单元测试。

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
| official packages | composition-lab 以无特权方式暴露 launch-plan 与 surface-graph capabilities | implemented |
| official packages | asset-lab 以无特权方式 preview assets 并生成需要审批的 import plans | implemented |
| official packages | projection-lab 以无特权方式生成 rebuild plans 并解释 source events | implemented |
| official packages | playable-seed 暴露 reference entry/play/Forge/assistant surfaces 以及需要审批的 edits | implemented |
| official packages | persona-lab 以无 kernel ontology 的方式 import 并 render persona profiles，且带 provenance | implemented |
| official packages | knowledge-lab normalize collections、match entries，并返回 plan-only injection output | implemented |
| official packages | context-lab 组装 generic blocks，包含 budget omissions 与 template rendering | implemented |
| official packages | text-transform-lab preview deterministic text transforms，包含 trace 与 validation diagnostics | implemented |
| official packages | assistant-lab 通过授权返回需要审批的 proposal | implemented |
| play-creation | 空白循环演练 assistant proposal、branch、asset、projection | implemented |
| proposals | 已批准的 proposal 可以执行通用 asset/projection 操作 | implemented |
| proposals | 被拒绝或未批准的 proposal 不能执行 | implemented |
| package authoring | 生成的 Python subprocess 包通过本地 conformance | implemented |
| package authoring | 生成的 TypeScript subprocess 包通过本地 conformance | implemented |
| package authoring | 生成的 experience 包 surface 通过本地 conformance | implemented |
| composition | 本地 composition 描述符验证包提供的 surface | implemented |

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
official.composition_lab                   PASS
official.asset_lab                         PASS
official.projection_lab                    PASS
official.playable_seed                     PASS
official.persona_lab                       PASS
official.knowledge_lab                     PASS
official.context_lab                       PASS
official.text_transform_lab                PASS
```

该套件应该以封闭失败为原则：任何列为 Platform Host Alpha 必需的用例必须通过，该里程碑才能被宣布完成。
