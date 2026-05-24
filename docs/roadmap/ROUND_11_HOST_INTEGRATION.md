# Round 11 — Host Integration（Install 真实管线 / 真实崩溃捕获 / 磁盘占用）

> [English](./ROUND_11_HOST_INTEGRATION.en.md) · [中文](./ROUND_11_HOST_INTEGRATION.md)

平台壳完工之后，Web shell 还在前端 mock 三个东西：Install 进度 prototype、Failure modal demo defaults、Disk usage 0 字节。Round 11 把这三处接到真实的 host/kernel 管线上，全部走公开协议、全部 plan-first、全部不引入 kernel.v1.install/crash/disk 这类内容化 ontology。

## 整体原则

```
✗ 不加 kernel.v1.install.*       (install 是普通能力包, 不属于 kernel)
✗ 不加 kernel.v1.crash.*         (失败是 project 生命周期, 用 project.failed 事件)
✗ 不加 kernel.v1.disk.*          (磁盘是包元数据, 加在 PackageRecord)

✓ install-lab 在已有协议上发"包命名空间"的进度事件 (package/install-lab/install.*)
✓ project 失败用 kernel/v1/project.failed (生命周期事件, 已有 project.* 命名空间)
✓ size_bytes 作为 PackageRecord 的一个字段, kernel.v1.package.list 返回时附带
```

## Phase A — Install 真实管线接通

### 问题

* `official/install-lab` 已经实现了 `resolve_plan / execute_plan / detect_kind / register_project / uninstall / list_installed / check_lockfile` 全套能力，但只有 `yg install` CLI 调用得到。
* Web `InstallModal` 当前 3 步流程纯前端 prototype：URL 输入 → 计划伪造 → 模拟进度。
* 没有进度事件，Web 无法显示真实的"克隆 X / 校验 Y / 写入 store"。

### 解决方案

1. 在 `InprocCapabilityInvoker` trait 加 `append_event`，对称于已有的 `invoke_capability` / `project_registry`。这给所有 inproc 包一个统一的"反向写事件"通道（带 principal、走 schema 校验、不绕过权限）。
2. install-lab 的 `resolve_plan` / `execute_plan` 在关键节点 emit 包命名空间事件：
   * `package/install-lab/install.plan.resolving`（开始解析）
   * `package/install-lab/install.plan.resolved`（解析完成，载荷含包数 / 权限摘要 / 签名摘要）
   * `package/install-lab/install.execute.started`（开始落盘）
   * `package/install-lab/install.execute.package.fetching`（per-package 拉取开始）
   * `package/install-lab/install.execute.package.fetched`（per-package 拉取完成）
   * `package/install-lab/install.execute.package.verified`（per-package 哈希/签名校验完成）
   * `package/install-lab/install.execute.completed`（写完 lockfile + profile + project）
   * `package/install-lab/install.execute.failed`（任意阶段失败，含原因）
3. 为这些事件 payload 写 JSON Schema 落 `docs/spec/v1/schemas/event/package.install-lab.*.schema.json`，并把它们登记进 `EVENT_KIND_REGISTRY`。
4. Web `InstallModal` 改造：
   * 步骤 1 提交 URL → 打开 session（kernel.v1.session.open）→ 调 `official/install-lab/resolve_plan` → 渲染真实包数 / 权限 / 签名摘要。
   * 步骤 2 用户审阅，按 Install → 调 `official/install-lab/execute_plan`，同时在另一根 SSE 上订阅 session 的 `package/install-lab/install.*` 事件流。
   * 步骤 3 进度由真实事件驱动（"clone X" / "verify Y" / "wrote lockfile"）。
   * 失败/取消按事件分支处理。
5. conformance 加用例：
   * `install_lab_emits_progress_events`（plan + execute 必须各自出现至少一个进度事件）
   * `install_lab_failure_emits_failed_event`（execute_plan 失败必须 emit `install.execute.failed`）

### 不做

* 不加 kernel.v1.install.* 协议方法。
* 不改 install-lab 的 CLI 路径行为（向后兼容）。
* 不实现 GPG signature smoke（受现有 `--require-signed` flag 已覆盖）。

## Phase B — 真实崩溃捕获

### 问题

* `SubprocessSupervisor` buffer stderr 但没有 ring buffer 上限，子进程死亡时只是 reverse pump break 或 invoke 报错。
* 没有 `kernel/v1/project.failed` 事件，项目崩溃后状态也不会自动转 Failed。
* `ProjectRegistry` 没有 last_failure 字段。
* Web `FailureModal` 当前是写死的 demo defaults（exit 137 / OOM / 假日志）。

### 解决方案

1. `SubprocessHandle` 加 stderr ring buffer：
   * 上限 64 KB（可配置，默认 64KB）
   * 按行收集，超限时丢弃最早的整行
   * `drain_recent_stderr() -> Vec<String>` 给监督器读最新尾部
2. `SubprocessSupervisor` 加子进程退出监听：
   * 当 `child.wait()` 返回非 0 / 信号时，捕获 `exit_code: Option<i32>`、`signal: Option<i32>`、`stderr_tail: Vec<String>`、`duration_ms`。
   * 通过 InprocCapabilityInvoker.append_event 反向写 `kernel/v1/project.failed` 事件（如果失败的包关联了项目）。
3. `ProjectRegistry` 加 `last_failure: Option<ProjectFailure>` 字段：
   ```rust
   struct ProjectFailure {
       at: DateTime<Utc>,
       exit_code: Option<i32>,
       signal: Option<i32>,
       stderr_tail: Vec<String>,
       duration_ms: u64,
       package_id: PackageId,
   }
   ```
4. `kernel.v1.project.list` 在 ProjectSummary 里附带 `last_failure: Option<ProjectFailureSummary>`（受 redaction 限制：stderr_tail 只暴露给 host_admin/host_dev）。
5. 加 `kernel.v1.project.failure` 方法（host_admin/host_dev 限定）返回完整失败详情。
6. Web `FailureModal` 改造：
   * 通过 `kernel.v1.project.failure` 读真实 exit_code/signal/stderr_tail
   * 不再写死 137/OOM
   * 没有失败记录时显示空状态而非伪造数据
7. conformance 加用例：
   * `subprocess_crash_emits_project_failed_event`
   * `project_failure_method_redacts_stderr_for_anonymous`
   * `project_failure_method_returns_full_data_for_host_admin`

### 不做

* 不加自动重启逻辑（用户手动决定）。
* 不持久化崩溃历史（only last failure；历史记录在 event log 里通过 list_events 查）。

## Phase C — Per-Package 磁盘占用

### 问题

* `PackageRecord` 没有 size_bytes 字段。
* Web Disk Usage 总是显示 0 字节。

### 解决方案

1. 新建 `crates/ygg-runtime/src/disk_usage.rs`：
   * `pub fn directory_size(path: &Path) -> std::io::Result<u64>` — 递归求和文件 size，软链不跟随
   * `pub fn package_disk_usage(store_dir: &Path, tree_hash: &str) -> std::io::Result<u64>`
2. `PackageRecord` 加 `size_bytes: Option<u64>` 字段（Optional 因为非 store 加载的包没有 store path）。
3. `runtime/packages.rs` 在 load 包时计算并缓存 size_bytes（cache 5 分钟，避免热路径反复 walk）。
4. Web `WorkshopUtilities` 的 `DiskSegment.bytes` 接 `PackageRecord.size_bytes`。
5. conformance 加 `package_record_includes_size_bytes`。

### 不做

* 不引入磁盘配额 / 警报。
* 不建立后台磁盘监控任务（按需计算 + cache 即可）。

## 推进顺序

每个 Phase 独立 commit + push。完成后整体汇报。

```
A → B → C
```

A 引入的 `InprocCapabilityInvoker.append_event` 是 B 的前置依赖（B 也要从 supervisor 反向写事件）。C 独立。

## 文档收敛（最后）

* 删除 `docs/roadmap/ROUND_11_HOST_INTEGRATION.{md,en.md}`（计划文档完工即弃）
* 更新 `docs/ALPHA_STATUS.{md,en.md}` Web shell + project + install 部分
* 更新 `docs/roadmap/NEXT_STEPS.{md,en.md}` 把这三件事从"deferred"移到"done"
* 更新 `docs/spec/v1/EVENT_KIND_REGISTRY.{md,en.md}` 登记新事件类型
* `clients/web/README.md` 更新 Install/Failure/Storage 数据接线说明
