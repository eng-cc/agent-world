# Runtime Sidecar Orphan / GC Fail-safe Evidence（2026-03-11）

审计轮次: 1

## Meta
- 日期: `2026-03-11`
- 执行角色: `runtime_engineer`
- 关联任务: `TASK-WORLD_RUNTIME-033 / T7.3`
- 关联 PRD-ID: `PRD-WORLD_RUNTIME-014/015`
- 结论: `pass`

## 背景
- T7.2 的真实 QA 样本在 `latest_retained_height=47` 时曾观测到瞬时 `orphan_blob_count=1`。
- 同一条样本链在 `latest_retained_height=65` 时已收敛到 `orphan_blob_count=0`，说明该信号更像窗口态而非稳定泄漏。

## 实现判断
- `storage_metrics` 的 `orphan_blob_count` 计算方式是：`execution_sidecar_blobs - pin_count`。
- `World::save_to_dir()` 内部会先把 sidecar snapshot/journal 分段写入 `.distfs-state/blobs`，随后才通过 `persist_sidecar_generation_index(...)` 刷新 generation index 并执行 `sweep_sidecar_orphan_blobs(...)`。
- 因此在 sidecar blob 已写入、但 generation index / GC 尚未完成的窗口里，采样可能看到瞬时 orphan；下一次成功 save/GC 后应收敛到 `0`。

## 自动化证据
- 新增测试：`storage_metrics::tests::collect_storage_metrics_sidecar_orphan_recovers_after_successful_save`
- 测试过程：
  - 先执行一次正常 `save_to_dir()` 建立 sidecar 基线。
  - 人工向 `.distfs-state/blobs` 注入一个未被 generation index 引用的 orphan blob。
  - 验证 `collect_storage_metrics(...)` 能观察到 `orphan_blob_count > 0`。
  - 再执行一次成功 `save_to_dir()`，验证 `sweep_sidecar_orphan_blobs(...)` 后 `orphan_blob_count = 0`、`last_gc_result = success`。

## 测试记录
- `env -u RUSTC_WRAPPER cargo test -p oasis7 --bin oasis7_chain_runtime collect_storage_metrics_sidecar_orphan_recovers_after_successful_save -- --nocapture`
- `env -u RUSTC_WRAPPER cargo test -p oasis7 --bin oasis7_chain_runtime collect_storage_metrics_reports_storage_snapshot -- --nocapture`

## 结论
- 当前没有证据表明 sidecar orphan 是稳定泄漏。
- T7.2 pre-checkpoint 样本中的 `orphan_blob_count=1` 可以按“采样命中 sidecar/GC 时序窗口”解释。
- T7.3 当前可视为已补齐一条可回归的 GC fail-safe 证据；后续若要继续收紧，可在 gate 或 QA 手册中明确“优先取 post-GC / 稳态样本”。
