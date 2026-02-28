# S10 Distfs Probe Bootstrap（2026-02-28）

## 目标
- 修复 S10 运行中 `distfs_total_checks=0` 导致 `metric_gate=insufficient_data` 的稳定告警。
- 保持正向门禁：不放宽阈值，而是让 runtime 产生可用 distfs 样本。

## 范围
- 代码：
  - `crates/agent_world/src/bin/world_chain_runtime/reward_runtime_worker.rs`
- 验证：
  - `scripts/s10-five-node-game-soak.sh` 发布基线命令复跑
- 文档：
  - `testing-manual.md`
  - `doc/devlog/2026-02-28.md`

## 非目标
- 不改动 distfs 校验算法与阈值语义。
- 不改动 S9/S10 的失败策略。

## 接口 / 数据
- 新增 runtime 内部行为：
  - reward worker 启动时检查 `storage_root/blobs`。
  - 当 blob 集为空时，自动写入最小 probe seed blob（幂等）。
- 预期效果：
  - `GET /v1/chain/status.reward_runtime.distfs_total_checks` 在 S10 长窗内可增长到 `>0`。
  - S10 `summary.json.run.metric_gate.status` 从 `insufficient_data` 变为 `pass`（若其他指标正常）。

## 里程碑
- M1：建档。
- M2：实现 probe seed bootstrap。
- M3：复跑 S10 发布基线并验证指标。
- M4：文档收口与结项。

## 风险
- seed blob 若重复写入可能增加无意义存储。
  - 缓解：仅在 blob 为空时写入，且内容固定幂等。
- 启动时写入失败可能引入噪声。
  - 缓解：错误写入 `last_error`，主流程不中断；后续仍可由真实业务数据补齐 probe。
