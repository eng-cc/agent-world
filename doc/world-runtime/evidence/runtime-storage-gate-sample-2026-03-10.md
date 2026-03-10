# Runtime Storage Gate 实测样本（2026-03-10）

审计轮次: 4

## Meta
- 样本 ID: `RT-STORAGE-GATE-SAMPLE-20260310-234544`
- 日期: `2026-03-10`
- 执行角色: `runtime_engineer`
- 关联任务: `TASK-WORLD_RUNTIME-033 / T7.2`
- 关联脚本: `scripts/world-runtime-storage-gate.sh`
- 总结论: `fail`

## 输入
- 运行命令: `env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_chain_runtime -- --node-id t72-node --world-id t72-world --storage-profile release_default --status-bind 127.0.0.1:5221 --execution-world-dir .tmp/runtime_t72_sample/execution-world --execution-records-dir .tmp/runtime_t72_sample/execution-records --storage-root .tmp/runtime_t72_sample/storage-root --execution-bridge-state .tmp/runtime_t72_sample/execution-bridge-state.json`
- 状态样本: `.tmp/runtime_t72_sample/artifacts/status.json`
- gate 摘要: `.tmp/world_runtime_storage_gate/20260310-234544/summary.md`（fresh sample） / `.tmp/world_runtime_storage_gate/20260310-234829/summary.md`（feedback 注入后）

## 关键观察
- `storage_profile=release_default`
- `effective_budget.profile=release_default`
- `checkpoint_count=0`
- `replay_summary.mode=full_log_only`
- feedback 注入后 `latest_retained_height=16`（仍未生成 checkpoint）
- `orphan_blob_count=0`
- `last_gc_result=success`
- `degraded_reason=null`

## 当前结论
- 本轮真实 `world_chain_runtime` 样本已成功接入 `scripts/world-runtime-storage-gate.sh`。
- gate 当前失败原因持续为 `checkpoint_count=0`；即使在 feedback 注入后 `latest_retained_height` 已推进到 `16`，仍保持 `full_log_only`，尚未形成 checkpoint 保留基线。
- 该结果符合当前专题状态：T7.2 入口已落地，但真实样本下的默认 profile 目录/指标差异仍需继续补齐。

## 后续动作
- 下一步优先补齐默认 profile 下 checkpoint 生成/保留样本，再复跑 gate。
- 若后续样本仍保持 `full_log_only`，需评估是阈值设置问题还是 checkpoint 生成链路未触发。

## 根因判断
- `crates/agent_world/src/bin/world_chain_runtime/execution_bridge.rs` 中 `maybe_persist_execution_checkpoint_for_record(...)` 明确要求 `record.height % checkpoint_interval_heights == 0` 才会落盘 checkpoint。
- `release_default` profile 的 `execution_checkpoint_interval=64`；当前真实样本在 feedback 注入后仅推进到 `latest_retained_height=16`，因此 `checkpoint_count=0` 更像“尚未达到阈值”，而不是 checkpoint 链路必然失效。
- 下一步优先方案不是盲修代码，而是把样本推进到 `height >= 64` 或引入显式 save/flush 场景，再复跑 gate。
