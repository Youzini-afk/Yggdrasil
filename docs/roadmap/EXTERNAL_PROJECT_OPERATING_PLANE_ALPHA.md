# External Project Operating Plane Alpha

> [English](./EXTERNAL_PROJECT_OPERATING_PLANE_ALPHA.en.md) · [中文](./EXTERNAL_PROJECT_OPERATING_PLANE_ALPHA.md)

这是临时执行计划。完成后删除，长期内容收敛到 `ALPHA_STATUS`、`NEXT_STEPS`、外部项目操作平面指南、conformance matrix 和相关包文档。

## 为什么现在做

Yggdrasil 不能只要求项目先适配 manifest/capability 契约，否则仍会像一个高级插件宿主。真实世界的大量项目只提供 git repo、npm package、local folder、CLI、dev server 或 Docker image。平台应能围绕未适配项目提供理解、运行计划、维护、修改、部署计划和 adapter 生成能力，同时保持内核内容无关。

外部调研保存在 `/tmp/opencode/ygg-external-project-plane-20260520/`。关键证据：

- GitHub 2026 supply-chain 资料强调近期开源攻击集中在 workflow 与 secret exfiltration，建议 pin actions、避免不安全触发、减少 secret 使用。
- npm v11 官方 scripts 文档确认 `npm install` / `npm ci` 会自动触发 `preinstall`、`install`、`postinstall`、`prepare` 等 lifecycle scripts；因此 install 等同执行不可信代码。
- Agent Sandbox / remote code execution sandbox 资料强调 untrusted code 必须隔离 filesystem、process、network、kernel，并配合 default-deny egress、资源限制、短生命周期和审计。

## 四类对象

1. **Ygg Package**：已适配能力提供者，有 manifest、capabilities、permissions、surface、conformance。
2. **External Project**：未适配外部项目引用，例如 git/npm/local/archive。默认不可信，不进入 package registry。
3. **Managed Workspace**：External Project 的受控实例，包含 source ref、revision、workspace state、plans、logs、entrypoints、patches、audit refs。它不是 kernel object，通过普通包拥有的 events/assets/projections 表达。
4. **Adapter / Wrapper Package**：把某个外部项目的稳定操作包装成普通 Ygg package/capability。只有 adapter/wrapper 才进入 capability 世界。

## 红线

- 不新增 `kernel.project.*`、`kernel.workspace.*`、`kernel.git.*`、`kernel.npm.*`、`kernel.deploy.*`、`kernel.ide.*`。
- External Project 不是 package；Managed Workspace 不是 kernel object。
- 未适配项目不得直接注册成 capability provider。
- 危险动作（clone/install/run/write/network/secrets/deploy）必须 plan-first、policy-checked、proposal/approval-gated、audited、redacted。
- Alpha 默认 no-network / no-execution；conformance 不依赖公网。
- 不默认执行 `npm install`、`pip install`、`cargo build`、`make` 或任意 project script。
- 不继承宿主 `.env`、SSH key、browser profile、home directory 或 raw secrets。
- Agent 不能拥有 shell；只能 draft plan/proposal/patch，执行必须由 host executor/policy 完成。
- UI 只走 public protocol，不读 workspace directory、SQLite、runtime internals，不直接操作进程。
- 不做 marketplace、billing、hosted deployment、full IDE、terminal emulator、cloud PaaS。

## Phase E0 — Plan, Research, ADR

目标：固化战略、外部证据、阶段边界和红线。

交付：

- 本临时双语计划。
- README / ALPHA_STATUS / NEXT_STEPS 当前主线更新。
- 外部 evidence 路径记录。

验收：doc links、diff check、工作区 clean 后 commit/push。

## Phase E1 — Project Intake Lab（no execution）— 已完成

目标：用户输入 git/npm/local/archive ref 后，Yggdrasil 可生成静态 intake report、stack guess、workspace plan、risk summary、candidate entrypoints 和 adapter plan；不 clone、不 install、不 run。

已交付：

- 普通官方包 `official/project-intake-lab`，`rust_inproc` manifest + surfaces。
- 能力：`describe_intake_contract`、`inspect_external_project_ref`、`detect_project_stack_from_metadata`、`draft_workspace_plan`、`draft_security_risk_summary`、`list_candidate_entrypoints`、`draft_adapter_plan`。
- inproc handler；raw-secret/path traversal/unsafe local path 阻断。
- fixtures：git/npm/local/static metadata 示例。
- conformance cases：no execution、source classification、node/rust/python/static/unknown detection、npm lifecycle risk flags、adapter plan。
- `profiles/forge-alpha.yaml` autoload。

验收：package check、workspace tests、conformance、no `kernel.project.*` residue。

## Phase E2 — Workspace Action Policy Boundary（deny-by-default fake executor）— 已完成

目标：建立危险 workspace action 的统一 policy/audit/proposal shape，但默认不真实执行。

已交付：

- 普通官方包 `official/workspace-lab`。
- 能力：`describe_workspace_contract`、`draft_workspace_creation`、`explain_required_permissions`、`request_workspace_action`、`summarize_workspace_audit`。
- action taxonomy：clone_project/read_metadata/install_dependencies/run_command/run_tests/stop_process/read_logs/discover_entrypoints/write_patch/deploy_plan。每个 action 标注 risk_level、requires_approval、executes_code、network_required、filesystem_write_required。
- 默认 `denied_by_default` / `requires_approval`，fake executor shape，不调用 host shell。
- Alpha 阶段不认可 approval_token；`approval_token_honored=false` 始终为 false。
- policy/action mismatch fail-closed；未知 action fail-closed。
- audit event shape（package-owned）与 redaction（不含 raw env/logs/commands/secrets）。
- conformance（7 个用例）：contract shape / action taxonomy deny-default / policy mismatch fail-closed / raw secret 阻断 / audit redacted / no forbidden namespace / no execution。

验收：default no execution；workspace 不进入 package registry；UI/public protocol 形状稳定。

## Phase E3 — Managed Workspace Deterministic Proof

目标：用 deterministic fixture 证明 workspace state/projection/log/entrypoint/patch flow，不做任意项目真实运行。

交付：

- `workspace-lab` 增加 deterministic fixture workspace 能力：`create_managed_workspace`、`inspect_workspace`、`read_workspace_metadata`、`plan_run`、`record_fixture_process_result`、`discover_workspace_entrypoints`、`draft_workspace_patch`。
- workspace state 以 package-owned events/assets/projections/proposals 表达。
- bounded redacted logs、opaque process_ref / workspace_ref。
- patch 只生成 proposal，不直接写入真实文件。
- examples fixture project（safe metadata only）。
- conformance：workspace projection、entrypoint discovery、bounded logs、patch proposal、no package registry pollution。

验收：仍无 arbitrary shell；无真实 install/run；所有 write 仍 proposal-gated。

## Phase E4 — Web Project Aggregation UI

目标：Home/Forge 能显示 External Projects / Managed Workspaces / risk / entrypoints / logs / adapter candidates，仍 public protocol-only。

交付：

- Home project/operating-plane card。
- Forge project intake panel、workspace cards、risk badges、entrypoints/logs/proposal previews。
- View-model/render helper，遵守 Performance & Code Health Web render discipline。
- No direct filesystem/process access。

验收：Web TypeScript；500-event/render discipline 不倒退；UI 不读 private runtime。

## Phase E5 — Adapter / Wrapper Generation Proof

目标：从 fixture workspace 生成可读、可检查、可替换的普通 adapter package 骨架。

交付：

- 普通官方包 `official/adapter-lab`。
- 能力：`describe_adapter_contract`、`draft_adapter_plan`、`infer_capability_candidates`、`generate_subprocess_wrapper`、`generate_manifest`、`generate_fixture`、`explain_adapter_permissions`、`export_adapter_package`。
- 最小 adapter：one command → one capability，subprocess wrapper，manifest，fixture，README。
- examples package / composition replacement proof。
- conformance：generated adapter package check passes，no official privilege，minimal permission declaration，inferred confidence 标注。

验收：adapter 是 ordinary package；不自动 publish；不自动授予 network/secrets。

## Phase E6 — Durable Docs Cleanup & Final Validation

目标：删除临时计划，收敛到长期指南和状态文档。

交付：

- 新增 `docs/guides/EXTERNAL_PROJECT_OPERATING_PLANE.md` 与 `.en.md`。
- README / ALPHA_STATUS / NEXT_STEPS / CONFORMANCE_MATRIX / package docs 收敛。
- 删除本临时计划。
- 记录外部 evidence sources。

最终验证：

- `cargo test --workspace`
- `cargo run -p ygg-cli -- conformance`
- `cargo run -p ygg-cli -- package check packages/official/project-intake-lab/manifest.yaml`
- `cargo run -p ygg-cli -- package check packages/official/workspace-lab/manifest.yaml`
- `cargo run -p ygg-cli -- package check packages/official/adapter-lab/manifest.yaml`
- `tsc -p clients/web/tsconfig.json --noEmit`
- markdown local links
- `git diff --check`
- temporary plan residue check
- forbidden kernel namespace residue check
