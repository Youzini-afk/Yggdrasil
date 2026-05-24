# 人测前稳定化临时计划

> 这是临时执行计划。Phase 4 完成后必须删除，并把长期有价值的内容收敛进 `docs/ALPHA_STATUS.md`、`docs/roadmap/NEXT_STEPS.md`、`docs/performance/` 与相关 guide。

本轮目标不是继续扩展平台表面积，而是在真实人测前把当前路径打稳：编译/CI 噪声、YdlTavern 安装启动闭环、surface 安全边界、secret/audit/failure 诊断、真实性能基线。

## 非目标

- 不新增 `kernel.v1.install.*`、`kernel.v1.crash.*`、`kernel.v1.disk.*`、`kernel.v1.model.*`、`kernel.v1.chat.*`、`kernel.v1.agent.*`。
- 不重设计 YdlTavern 内部 UI。
- 不继续按文件大小拆大文件。
- 不做 WIT/WASM、remote package、Powerbox、marketplace。
- 不为了性能绕过权限、schema、hook、redaction、audit 或 public protocol。

## Phase 0 — 当前基线与计划

建立当前 `main` 的事实基线，明确旧报告中 `InstallArgs` / `UninstallArgs` 缺字段已过时，当前真正要清的是 warning 噪声、安装启动闭环与人测热路径。

成功标准：计划写入、提交、push；后续 Phase 4 删除。

## Phase 1 — CI / warning truth pass

清理当前真实编译警告：

- `unused_imports`。
- `private_interfaces`。
- `ygg-cli` lib/bin 结构导致的 `dead_code` 噪声分层处理：真死代码删除，CLI 命令模块的结构性噪声局部解释，不做全局 blanket allow。

验证：

```bash
cargo check -p ygg-core -p ygg-runtime -p ygg-service -p ygg-cli --all-targets
cargo test -p ygg-core -p ygg-runtime -p ygg-service -p ygg-cli --all-targets
./scripts/validate-schemas.sh
cargo run -p ygg-cli -- conformance --fail-fast --slowest 20
npm run build --prefix clients/web
```

## Phase 2 — YdlTavern install → project registry → launch/surface bundle 闭环

从 clean data dir 验证并修通：

```text
yg install ../YdlTavern
→ detect project.yaml
→ register project
→ kernel.v1.project.list 可见
→ kernel.v1.project.start 返回 session_id
→ kernel.v1.surface.resolve_bundle 返回 installed project bundle
→ /surface-bundles/projects/{project_id}/bundle.mjs 可访问
→ iframe surface 能 mount
```

重点：

- `install-lab` 需要正确处理 project root，而不只找 package manifest。
- YdlTavern surface build 产物必须进入 host 可服务目录。
- surface-only package 不应因为 `entry.kind: wasm` placeholder 被误判 degraded。

## Phase 3 — Surface RPC / secret / audit / failure diagnostics 边界硬化

在人测前保证 surface 可以用，但不能拿到无限 host-dev RPC。

重点：

- iframe bridge 最小 allowlist / gate。
- Failure diagnostics 只暴露 redacted 信息。
- raw secret 不进入 UI state、event、audit、log、clipboard。
- YdlTavern live model 仍只走 `secret_ref` + public outbound。

验证：

```bash
cargo run -p ygg-cli -- conformance --tag surface --tag outbound --tag secret --tag stream --tag permission --fail-fast --slowest 20
```

## Phase 4 — 真实性能基线轮 + 文档收敛

测真实人测路径，不做盲目优化：

- host startup / profile autoload。
- Home `project.list` + `storage_summary`。
- project start → iframe ready。
- YdlTavern surface mount / message formatting。
- outbound stream TTFT / audit overhead。

若发现 `storage_summary` 递归扫目录阻塞 Home，只能做 bounded / cached / unknown fallback，不能新增 `kernel.v1.disk.*`，不能让 UI 直接扫文件系统。

最后删除本临时计划，把长期内容收敛进状态、性能与 guide 文档。
