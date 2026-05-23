# External Project Operating Plane Guide

> [English](./EXTERNAL_PROJECT_OPERATING_PLANE.en.md) · [中文](./EXTERNAL_PROJECT_OPERATING_PLANE.md)

External Project Operating Plane shows that Yggdrasil does not have to accept only projects that already implement the manifest and capability contract. Unadapted git, npm, local, or archive projects can first be understood, risk-scored, planned, displayed, and wrapped by the platform. Only stable adapters or wrappers enter the ordinary Ygg package and capability world.

External evidence is saved under `/tmp/opencode/ygg-external-project-plane-20260520/`. This work used GitHub supply-chain security material, npm lifecycle script documentation, and agent/RCE sandbox references. The core conclusion is that install/run means executing untrusted code. Workflow and secret exfiltration are real risks. Unadapted projects must go through planning, policy, proposal, and audit boundaries first.

## Four object classes

| Object | Meaning | Enters capability registry |
|---|---|---:|
| Ygg Package | Adapted provider with manifest, capabilities, permissions, surfaces, and conformance. | Yes |
| External Project | Unadapted reference such as git/npm/local/archive. Untrusted by default. | No |
| Managed Workspace | Controlled instance/plan/fixture around an External Project, including source ref, workspace state, entrypoints, patch proposals, and audit refs. Not a kernel object. | No |
| Adapter / Wrapper Package | Ordinary Ygg package/capability wrapper around stable external-project operations. | Yes |

This avoids the old plugin-host trap. An external project can remain unchanged while Yggdrasil performs intake, workspace planning, risk summaries, project aggregation UI, patch proposals, and adapter previews around it.

## Implemented packages

### `official/project-intake-lab`

Ordinary official package, no kernel privilege. It exposes these capabilities:

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

Capability boundaries:

- Static intake, metadata-based stack detection, risk summaries, workspace/adapter planning only.
- No clone, no install, no run, no network, no local filesystem read.
- Blocks raw secrets, path traversal, home paths, and sensitive absolute local paths.
- Detects npm lifecycle scripts (`preinstall`, `install`, `postinstall`, `prepare`, `prepublish`) as `executes_code` / `requires_approval`.
- Adapter previews must use ordinary third-party package ids, never `official/`, and reject path traversal or unsafe characters.
- Capability ids must belong to the adapter package namespace.
- Produces manifest/wrapper/fixture/readiness previews only. No file write, execution, or publishing.

### `official/workspace-lab`

Ordinary official package, no kernel privilege. It exposes 12 capabilities:

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

Capability boundaries:

- Action taxonomy covers `clone_project`, `read_metadata`, `install_dependencies`, `run_command`, `run_tests`, `stop_process`, `read_logs`, `discover_entrypoints`, `write_patch`, and `deploy_plan`.
- Each action carries `risk_level`, `requires_approval`, `executes_code`, `network_required`, and `filesystem_write_required`.
- `request_workspace_action` denies by default. Policy/action mismatches fail closed.
- Deterministic fixture workspace capabilities prove workspace descriptors, entrypoints, run plans, fixture results, and patch proposal shapes without creating directories, spawning processes, or reading files.
- Patch output is proposal-only with `file_write_performed=false`.

## Web aggregation entry

`clients/web/src/projects/external-projects.ts` aggregates no-execution outputs from `project-intake-lab` and `workspace-lab` through public protocol/capability invoke.

- Home/Play displays an External Project Operating Plane rail.
- Forge displays an External Projects / Managed Workspaces panel.
- Assistant drawer displays lightweight inspect / draft patch / generate adapter plan entries.
- UI does not read SQLite, runtime internals, local project directories, or process state.

## Security red lines

- Do not add `kernel.v1.project.*`, `kernel.v1.workspace.*`, `kernel.v1.git.*`, `kernel.v1.npm.*`, `kernel.v1.deploy.*`, or `kernel.v1.ide.*`.
- External Project is not a package; Managed Workspace is not a kernel object; Adapter/Wrapper is the package path.
- Unadapted projects do not directly register as capability providers.
- Dangerous actions must be plan-first, policy-checked, proposal/approval-gated, audited, and redacted.
- Do not execute `npm install`, `pip install`, `cargo build`, `make`, or arbitrary project scripts by default.
- Do not inherit host `.env`, SSH keys, browser profiles, home directories, or raw secrets.
- Agents may draft plans/proposals/patches only; execution must go through host executor/policy.
- Web shell remains public-protocol-only.

## Example

`examples/packages/external-project-adapter-preview/manifest.yaml` is the E5 adapter preview fixture. It uses the `thirdparty/example-adapter` namespace and proves that external-project adapters should use the same package path as every other package. It is not a published artifact, does not write to a user project, and does not execute external commands.

Check it with:

```bash
cargo run -p ygg-cli -- package check packages/official/project-intake-lab/manifest.yaml
cargo run -p ygg-cli -- package check packages/official/workspace-lab/manifest.yaml
cargo run -p ygg-cli -- package check examples/packages/external-project-adapter-preview/manifest.yaml
cargo run -p ygg-cli -- conformance --tag project_intake
cargo run -p ygg-cli -- conformance --tag workspace_lab
```

## Next directions

The current capabilities deliberately stop at plans and previews. Real deployment and maintenance need more boundaries:

- Host-controlled sandbox/workspace executor.
- Real clone/install/run/test/stop/log execution boundaries.
- Per-action approval, resource limits, egress policy, env allowlists, process lifecycle, artifact cleanup.
- Patch apply / test rerun / deployment preview through branch/proposal flows.
- Deeper project graph and dependency risk analysis.

These should still proceed as ordinary package and host-executor substrate, not as kernel product ontology.
