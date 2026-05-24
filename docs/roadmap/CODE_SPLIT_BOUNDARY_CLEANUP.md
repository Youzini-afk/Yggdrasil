# Code Split & Boundary Cleanup（临时执行计划）

> 本文是执行期计划。全部 Phase 完成后删除，并把长期结论收敛到 `ALPHA_STATUS`、`NEXT_STEPS` 与相关 guide。

目标：拆分近期膨胀的运行时、安装、Web install、Home、conformance registry、schema exporter 文件。以 move-only / behavior-preserving 为主，不改公开协议形状，不新增 kernel 内容 ontology。

## Baseline

- Conformance: `427 cases` pass.
- Web build: pass；main chunk 约 `447 KB` / gzip `143 KB`。
- Redline grep: Rust crates 中无 `kernel.v1.install` / `kernel.v1.crash` / `kernel.v1.disk`。
- Schema count: 115 schemas（methods 63, events 45）。

## Red lines

- 不新增 `kernel.v1.install.*`、`kernel.v1.crash.*`、`kernel.v1.disk.*`。
- Install 仍是普通能力包：`official/install-lab/*` 经 `kernel.v1.capability.invoke` 调用。
- Failure diagnostics 只用 redacted package/project summary，不暴露 raw stderr 或 host absolute path。
- Disk usage 只通过 project `storage_summary` 暴露聚合数字，不让 Web UI 读文件系统。
- 不手改 generated SDK；只改 generator。

## Phases

### S1 — Runtime protocol dispatch split

把 `crates/ygg-runtime/src/runtime/protocol_dispatch.rs` 拆为 facade + `runtime/protocol/*` domain modules。保持 `KernelMethod` 路由和 JSON shape 不变。

### S2 — Install backend split

把 `crates/ygg-runtime/src/inproc/install_lab.rs` 拆为 `install_lab/` 目录：types/source/planner/executor/layout/project_register/events/fs_copy。先以 move-only 为主。

### S3 — Web InstallModal split

把 `clients/web/src/components/install/install-modal.tsx` 拆为 modal shell、state hook、capability adapter 与 step components。

### S4 — Home route split

把 `clients/web/src/routes/home.tsx` 拆为 page composition + hooks/helpers：projects/timeline/disk/failure diagnostics/filtering/actions。

### S5 — Conformance registry split

把 `crates/ygg-cli/src/conformance/mod.rs` 的巨型 case registry 拆到 `conformance/registry/*`，runner 行为不变。

### S6 — Schema exporter split + docs convergence

把 `crates/ygg-cli/src/bin/export-schemas.rs` 拆到 reusable exporter modules；删除本计划，更新长期文档并最终验证。

## Verification per phase

- `cargo run -p ygg-cli -- conformance --fail-fast`
- `npm run build --prefix clients/web`（涉及 Web 时）
- `./scripts/validate-schemas.sh`（涉及 protocol/schema 时）
- `git diff --check`
- 红线 grep：`kernel.v1.install|kernel.v1.crash|kernel.v1.disk`
