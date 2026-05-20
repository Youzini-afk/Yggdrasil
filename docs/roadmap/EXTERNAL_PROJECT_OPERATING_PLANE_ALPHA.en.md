# External Project Operating Plane Alpha

> [English](./EXTERNAL_PROJECT_OPERATING_PLANE_ALPHA.en.md) · [中文](./EXTERNAL_PROJECT_OPERATING_PLANE_ALPHA.md)

This is a temporary execution plan. Delete it when complete and converge durable content into `ALPHA_STATUS`, `NEXT_STEPS`, the external project operating-plane guide, the conformance matrix, and package docs.

## Why now

Yggdrasil must not require every project to first adapt to its manifest/capability contract; otherwise it remains an advanced plugin host. Real projects often expose only git repositories, npm packages, local folders, CLIs, dev servers, or Docker images. The platform should help users understand, plan, maintain, modify, deploy-plan, and wrap unadapted projects while keeping the kernel content-free.

External evidence is saved under `/tmp/opencode/ygg-external-project-plane-20260520/`. Key evidence:

- GitHub 2026 supply-chain material emphasizes workflow compromise and secret exfiltration, recommending pinned actions, safer triggers, and reduced secret exposure.
- npm v11 official scripts documentation confirms `npm install` / `npm ci` automatically run lifecycle scripts such as `preinstall`, `install`, `postinstall`, and `prepare`; install therefore means executing untrusted code.
- Agent Sandbox / remote code execution sandbox material emphasizes filesystem, process, network, and kernel isolation, default-deny egress, resource limits, short lifecycles, and audit.

## Four object categories

1. **Ygg Package**: adapted provider with manifest, capabilities, permissions, surfaces, and conformance.
2. **External Project**: unadapted source reference such as git/npm/local/archive. It is untrusted by default and does not enter the package registry.
3. **Managed Workspace**: controlled instance of an External Project with source ref, revision, workspace state, plans, logs, entrypoints, patches, and audit refs. It is not a kernel object; ordinary packages express it through package-owned events/assets/projections.
4. **Adapter / Wrapper Package**: ordinary Ygg package/capability wrapper for stable external-project operations. Only adapters/wrappers enter the capability world.

## Red lines

- Do not add `kernel.project.*`, `kernel.workspace.*`, `kernel.git.*`, `kernel.npm.*`, `kernel.deploy.*`, or `kernel.ide.*`.
- External Project is not a package; Managed Workspace is not a kernel object.
- Unadapted projects must not be registered directly as capability providers.
- Dangerous actions (clone/install/run/write/network/secrets/deploy) must be plan-first, policy-checked, proposal/approval-gated, audited, and redacted.
- Alpha defaults to no-network / no-execution; conformance must not depend on the public internet.
- Do not automatically run `npm install`, `pip install`, `cargo build`, `make`, or arbitrary project scripts.
- Do not inherit host `.env`, SSH keys, browser profiles, home directories, or raw secrets.
- Agents do not get a shell; they draft plans/proposals/patches, while execution goes through host executor/policy.
- UI uses public protocol only; it must not read workspace directories, SQLite, runtime internals, or directly manage processes.
- No marketplace, billing, hosted deployment, full IDE, terminal emulator, or cloud PaaS.

## Phase E0 — Plan, Research, ADR

Goal: lock strategy, external evidence, phase boundaries, and red lines.

Deliverables: this temporary bilingual plan, README / ALPHA_STATUS / NEXT_STEPS status updates, and evidence path references.

Acceptance: doc links, diff check, clean commit/push.

## Phase E1 — Project Intake Lab (no execution) — COMPLETE

Goal: accept git/npm/local/archive refs and produce static intake report, stack guess, workspace plan, risk summary, candidate entrypoints, and adapter plan without clone/install/run.

Deliverables:

- Ordinary official package `official/project-intake-lab`, `rust_inproc` manifest + surfaces.
- Capabilities: `describe_intake_contract`, `inspect_external_project_ref`, `detect_project_stack_from_metadata`, `draft_workspace_plan`, `draft_security_risk_summary`, `list_candidate_entrypoints`, `draft_adapter_plan`.
- In-process handler; raw-secret/path-traversal/unsafe-local-path blocking.
- Fixtures for git/npm/local/static metadata.
- Conformance cases for no execution, source classification, node/rust/python/static/unknown detection, npm lifecycle risk flags, and adapter plan.
- `profiles/forge-alpha.yaml` autoload.

Acceptance: package check, workspace tests, conformance, and no `kernel.project.*` residue.

## Phase E2 — Workspace Action Policy Boundary (deny-by-default fake executor) — COMPLETE

Goal: define policy/audit/proposal shapes for dangerous workspace actions, still without real execution by default.

Deliverables:

- Ordinary official package `official/workspace-lab`.
- Capabilities: `describe_workspace_contract`, `draft_workspace_creation`, `explain_required_permissions`, `request_workspace_action`, `summarize_workspace_audit`.
- Action taxonomy: clone_project/read_metadata/install_dependencies/run_command/run_tests/stop_process/read_logs/discover_entrypoints/write_patch/deploy_plan. Each action annotated with risk_level, requires_approval, executes_code, network_required, filesystem_write_required.
- Default `denied_by_default` / `requires_approval`; fake executor shape; no host shell.
- Approval tokens not honored in Alpha; `approval_token_honored=false` always.
- Policy/action mismatch fail-closed; unknown action fail-closed.
- Package-owned audit event shape and redaction (no raw env/logs/commands/secrets).
- Conformance for unapproved dangerous action not executing (7 cases: contract shape / action taxonomy deny-default / policy mismatch fail-closed / raw-secret blocked / audit redacted / no forbidden namespace / no execution).

Acceptance: default no execution; workspace does not enter package registry; UI/public-protocol shape stable.

## Phase E3 — Managed Workspace Deterministic Proof — COMPLETE

Goal: prove workspace state/projection/log/entrypoint/patch flow with deterministic fixtures, not arbitrary real project execution.

Deliverables:

- `workspace-lab` fixture capabilities: `create_fixture_workspace`, `inspect_workspace`, `read_workspace_metadata`, `plan_workspace_run`, `record_fixture_process_result`, `discover_workspace_entrypoints`, `draft_workspace_patch`.
- Workspace state through package-owned events/assets/projections/proposals.
- Bounded redacted logs, opaque process_ref / workspace_ref.
- Patch as proposal only; no direct real-file writes.
- Safe metadata-only example fixture project.
- Conformance for workspace projection, entrypoint discovery, bounded logs, patch proposal, and no package-registry pollution.

Acceptance: still no arbitrary shell; no real install/run; all writes proposal-gated.

## Phase E4 — Web Project Aggregation UI

Goal: Home/Forge display External Projects / Managed Workspaces / risk / entrypoints / logs / adapter candidates, still public-protocol-only.

Deliverables: Home operating-plane card, Forge intake panel, workspace cards, risk badges, entrypoint/log/proposal previews, and view-model/render helpers that follow Performance & Code Health render discipline.

Acceptance: Web TypeScript; no render-discipline regression; UI does not read private runtime state.

## Phase E5 — Adapter / Wrapper Generation Proof

Goal: generate readable, checkable, replaceable ordinary adapter package skeletons from fixture workspaces.

Deliverables:

- Ordinary official package `official/adapter-lab`.
- Capabilities: `describe_adapter_contract`, `draft_adapter_plan`, `infer_capability_candidates`, `generate_subprocess_wrapper`, `generate_manifest`, `generate_fixture`, `explain_adapter_permissions`, `export_adapter_package`.
- Minimal adapter: one command → one capability, subprocess wrapper, manifest, fixture, README.
- Example package / composition replacement proof.
- Conformance: generated adapter package check passes, no official privilege, minimal permissions, inferred confidence labels.

Acceptance: adapter is an ordinary package; no auto-publish; no automatic network/secret grants.

## Phase E6 — Durable Docs Cleanup & Final Validation

Goal: delete this temporary plan and converge durable docs.

Deliverables: `docs/guides/EXTERNAL_PROJECT_OPERATING_PLANE.md` and `.en.md`, README / ALPHA_STATUS / NEXT_STEPS / CONFORMANCE_MATRIX / package docs updates, deletion of this plan, and evidence-source notes.

Final validation: workspace tests, conformance, package checks for the new labs, Web TypeScript, doc links, diff check, temporary-plan residue check, and forbidden-kernel-namespace residue check.
