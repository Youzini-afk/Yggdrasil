# 外部项目操作平面指南

> [English](./EXTERNAL_PROJECT_OPERATING_PLANE.en.md) · [中文](./EXTERNAL_PROJECT_OPERATING_PLANE.md)

External Project Operating Plane 说明 Yggdrasil 不必只接入已经适配清单和能力契约的项目。未适配的 git、npm、本地目录或 archive 项目可以先作为 external project 被平台理解、评估风险、规划、展示和包装。只有稳定的 adapter 或 wrapper 才进入普通 Ygg 包和能力体系。

外部调研证据保存在 `/tmp/opencode/ygg-external-project-plane-20260520/`。本阶段参考了 GitHub 供应链安全资料、npm lifecycle scripts 文档，以及 agent/RCE 沙箱资料。关键结论是：install/run 等于执行不可信代码。workflow 和 secret 泄漏是真实风险。未适配项目必须先走计划、策略、提案和审计边界。

## 四类对象

| 对象 | 含义 | 是否进入 capability registry |
|---|---|---:|
| Ygg Package | 已适配的能力提供者，有清单、能力、权限、surface 和检查。 | 是 |
| External Project | 未适配项目引用，例如 git/npm/local/archive。默认不可信。 | 否 |
| Managed Workspace | External Project 的受控实例、计划或 fixture，包含 source ref、workspace state、entrypoint、patch proposal 和 audit ref。它不是内核对象。 | 否 |
| Adapter / Wrapper Package | 把外部项目的稳定操作包装成普通 Ygg 包和能力。 | 是 |

这让平台避免退回“所有项目必须先写插件”的旧模式。外部项目可以保持原样。Yggdrasil 围绕它做 intake、workspace plan、风险摘要、项目聚合 UI、patch proposal 和 adapter preview。

## 已实现包

### `official/project-intake-lab`

普通官方包，无内核特权。提供以下能力：

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

- 只做静态 intake、基于元数据的技术栈检测、风险摘要和 workspace/adapter planning。
- 不 clone、不 install、不 run、不联网、不读本地文件系统。
- 阻断 raw secret、path traversal、home path 和敏感绝对本地路径。
- 检测 npm 生命周期脚本：`preinstall`、`install`、`postinstall`、`prepare`、`prepublish` 等会标记为 `executes_code` / `requires_approval`。
- Adapter preview 必须使用普通 third-party package id。不允许 `official/`，也不允许 path traversal 或 unsafe chars。
- 能力 id 必须属于 adapter package namespace。
- 只生成清单、wrapper、fixture 和 readiness preview。不写文件、不执行、不发布。

### `official/workspace-lab`

普通官方包，无内核特权。提供以下能力：

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
- `request_workspace_action` 默认拒绝。policy/action 不匹配时 fail-closed。
- Fixture workspace 能力证明 workspace descriptor、entrypoint、run plan、fixture result 和 patch proposal 的形状。它不创建目录、不启动进程、不读取文件。
- Patch 只生成提案形状，`file_write_performed=false`。

## Web 聚合入口

`clients/web/src/projects/external-projects.ts` 通过公开协议和能力调用聚合 `project-intake-lab` 与 `workspace-lab` 的计划输出。

- Home/Play 显示 External Project Operating Plane rail。
- Forge 显示 External Projects / Managed Workspaces panel。
- Assistant drawer 显示 inspect / draft patch / generate adapter plan 的轻量入口。
- UI 不读 SQLite、runtime internals、本地项目目录或进程状态。

## 安全红线

- 不新增 `kernel.project.*`、`kernel.workspace.*`、`kernel.git.*`、`kernel.npm.*`、`kernel.deploy.*`、`kernel.ide.*`。
- External Project 不是包；Managed Workspace 不是内核对象；Adapter/Wrapper 才是包。
- 未适配项目不得直接注册为 capability provider。
- 危险动作必须先计划，再通过策略、提案、审批、审计和脱敏边界。
- 默认不执行 `npm install`、`pip install`、`cargo build`、`make` 或任意 project script。
- 不继承宿主 `.env`、SSH key、browser profile、home directory 或 raw secrets。
- Agent 只能草拟计划、提案和 patch；执行必须由 host executor/policy 完成。
- Web shell 只走公开协议。

## 示例

`examples/packages/external-project-adapter-preview/manifest.yaml` 是 adapter preview fixture。它使用 `thirdparty/example-adapter` namespace，通过普通包清单证明外部项目 adapter 应走同一条包路径。它不是发布物，不自动写入用户项目，也不自动执行外部命令。

可检查：

```bash
cargo run -p ygg-cli -- package check packages/official/project-intake-lab/manifest.yaml
cargo run -p ygg-cli -- package check packages/official/workspace-lab/manifest.yaml
cargo run -p ygg-cli -- package check examples/packages/external-project-adapter-preview/manifest.yaml
cargo run -p ygg-cli -- conformance --tag project_intake
cargo run -p ygg-cli -- conformance --tag workspace_lab
```

## 后续方向

当前能力刻意停在计划和预览。后续若要进入真实部署与维护，需要补上：

- host 控制的沙箱和 workspace executor。
- clone/install/run/test/stop/logs 的真实执行边界。
- 单动作审批、资源限制、egress policy、env allowlist、进程生命周期和 artifact cleanup。
- patch apply / test rerun / deployment preview 的分支和提案流程。
- 更深入的 project graph 和 dependency risk analysis。

这些仍应作为普通包和 host executor 底座推进，不应进入内核 product ontology。
