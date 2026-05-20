# 外部项目操作平面指南

> [English](./EXTERNAL_PROJECT_OPERATING_PLANE.en.md) · [中文](./EXTERNAL_PROJECT_OPERATING_PLANE.md)

External Project Operating Plane Alpha 证明了 Yggdrasil 不必只接入已经适配 manifest/capability 契约的项目。未适配的 git/npm/local/archive 项目可以先作为 external project 被平台理解、风险评估、规划、展示和包装；只有稳定 adapter/wrapper 才进入普通 Ygg package/capability 世界。

外部调研证据保存在 `/tmp/opencode/ygg-external-project-plane-20260520/`。本阶段参考了 GitHub 供应链安全资料、npm lifecycle scripts 文档，以及 agent/RCE sandbox 资料。关键结论：install/run 等于执行不可信代码；workflow/secret exfiltration 是真实风险；未适配项目必须先 plan-first、default-deny、policy/proposal/audit gated。

## 四类对象

| 对象 | 含义 | 是否进入 capability registry |
|---|---|---:|
| Ygg Package | 已适配的能力提供者，有 manifest、capabilities、permissions、surfaces、conformance。 | 是 |
| External Project | 未适配项目引用，例如 git/npm/local/archive。默认不可信。 | 否 |
| Managed Workspace | External Project 的受控实例/计划/fixture，包含 source ref、workspace state、entrypoints、patch proposals、audit refs。它不是 kernel object。 | 否 |
| Adapter / Wrapper Package | 把外部项目稳定操作包装成普通 Ygg package/capability。 | 是 |

这让平台避免退回“所有项目必须先写插件”的旧模式：外部项目可以保持原样，Yggdrasil 围绕它做 intake、workspace plan、risk summary、project aggregation UI、patch proposal 和 adapter preview。

## 已实现包

### `official/project-intake-lab`

普通官方包，无内核特权。提供 11 项能力：

- `describe_intake_contract`
- `inspect_external_project_ref`
- `detect_project_stack_from_metadata`
- `draft_workspace_plan`
- `draft_security_risk_summary`
- `list_candidate_entrypoints`
- `draft_adapter_plan`
- `generate_adapter_manifest_preview`
- `generate_subprocess_wrapper_preview`
- `generate_adapter_fixture_preview`
- `check_adapter_readiness`

能力边界：

- 只做静态 intake、metadata-based stack detection、risk summary、workspace/adapter planning。
- 不 clone、不 install、不 run、不联网、不读本地文件系统。
- 阻断 raw secrets、path traversal、home path、敏感 absolute local path。
- 检测 npm lifecycle scripts：`preinstall`、`install`、`postinstall`、`prepare`、`prepublish` 等标记为 `executes_code` / `requires_approval`。
- Adapter preview 必须使用普通 third-party package id，不允许 `official/`，不允许 path traversal 或 unsafe chars。
- Capability id 必须属于 adapter package namespace。
- 生成 manifest/wrapper/fixture/readiness preview，不写文件、不执行、不发布。

### `official/workspace-lab`

普通官方包，无内核特权。提供 12 项能力：

- `describe_workspace_contract`
- `draft_workspace_creation`
- `explain_required_permissions`
- `request_workspace_action`
- `summarize_workspace_audit`
- `create_fixture_workspace`
- `inspect_workspace`
- `read_workspace_metadata`
- `plan_workspace_run`
- `record_fixture_process_result`
- `discover_workspace_entrypoints`
- `draft_workspace_patch`

能力边界：

- Action taxonomy 覆盖 `clone_project`、`read_metadata`、`install_dependencies`、`run_command`、`run_tests`、`stop_process`、`read_logs`、`discover_entrypoints`、`write_patch`、`deploy_plan`。
- 每个 action 标注 `risk_level`、`requires_approval`、`executes_code`、`network_required`、`filesystem_write_required`。
- `request_workspace_action` 默认 deny-by-default；Alpha 不认可 approval token；policy/action mismatch fail-closed。
- Deterministic fixture workspace 能力证明 workspace descriptor、entrypoints、run plan、fixture result、patch proposal 形状，但不创建目录、不启动进程、不读取文件。
- Patch 只生成 proposal shape，`file_write_performed=false`。

## Web 聚合入口

`clients/web/src/projects/external-projects.ts` 通过 public protocol/capability invoke 聚合 `project-intake-lab` 与 `workspace-lab` 的 no-execution 输出。

- Home/Play 显示 External Project Operating Plane rail。
- Forge 显示 External Projects / Managed Workspaces panel。
- Assistant drawer 显示 inspect / draft patch / generate adapter plan 的轻量入口。
- UI 不读 SQLite、runtime internals、本地项目目录或进程状态。

## 安全红线

- 不新增 `kernel.project.*`、`kernel.workspace.*`、`kernel.git.*`、`kernel.npm.*`、`kernel.deploy.*`、`kernel.ide.*`。
- External Project 不是 package；Managed Workspace 不是 kernel object；Adapter/Wrapper 才是 package。
- 未适配项目不得直接注册为 capability provider。
- 危险动作必须 plan-first、policy-checked、proposal/approval-gated、audited、redacted。
- 默认不执行 `npm install`、`pip install`、`cargo build`、`make` 或任意 project script。
- 不继承宿主 `.env`、SSH key、browser profile、home directory 或 raw secrets。
- Agent 只能 draft plan/proposal/patch；执行必须由 host executor/policy 完成。
- Web shell 只走 public protocol。

## 示例

`examples/packages/external-project-adapter-preview/manifest.yaml` 是 E5 的 adapter preview fixture。它使用 `thirdparty/example-adapter` namespace，通过普通 package manifest 证明外部项目 adapter 应走同一条 package path。它不是发布物，不自动写入用户项目，也不自动执行外部命令。

可检查：

```bash
cargo run -p ygg-cli -- package check packages/official/project-intake-lab/manifest.yaml
cargo run -p ygg-cli -- package check packages/official/workspace-lab/manifest.yaml
cargo run -p ygg-cli -- package check examples/packages/external-project-adapter-preview/manifest.yaml
cargo run -p ygg-cli -- conformance --tag project_intake
cargo run -p ygg-cli -- conformance --tag workspace_lab
```

## Conformance

External Project Operating Plane Alpha 结束时：

- 全仓 `cargo run -p ygg-cli -- conformance`：275 个具名 cases。
- `project_intake`：16 个 cases。
- `workspace_lab`：14 个 cases。

覆盖：contract shape、source classification、stack detection、npm lifecycle risk、workspace plan no execution、local path rejection、adapter plan no execution、adapter manifest preview no write、official/path traversal/capability namespace rejection、wrapper preview no execution、fixture redaction、readiness checklist、workspace action deny-default、policy mismatch fail-closed、audit redaction、fixture workspace、entrypoint discovery、patch proposal、raw-secret blocking 和 forbidden namespace blocking。

## 后续方向

本阶段刻意停在 no-execution / no-network / deterministic preview。后续若要进入真实部署与维护，需要另起阶段：

- host-controlled sandbox/workspace executor。
- clone/install/run/test/stop/logs 的真实执行边界。
- per-action approval、resource limit、egress policy、env allowlist、process lifecycle、artifact cleanup。
- patch apply / test rerun / deployment preview 的 branch/proposal 流程。
- 更深入的 project graph 和 dependency risk analysis。

这些仍应作为普通 package / host executor substrate 推进，不应进入 kernel product ontology。
