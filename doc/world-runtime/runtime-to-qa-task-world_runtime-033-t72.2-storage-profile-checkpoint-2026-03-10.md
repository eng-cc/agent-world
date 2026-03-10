# Role Handoff Detailed

## Meta
- Handoff ID: `HANDOFF-WORLD_RUNTIME-033-T7.2-2026-03-10-CHKPT-PROFILE`
- Date: `2026-03-10`
- From Role: `runtime_engineer`
- To Role: `qa_engineer`
- Related Module: `world-runtime`
- Related PRD-ID: `PRD-WORLD_RUNTIME-014/015`
- Related Task ID: `TASK-WORLD_RUNTIME-033 / T7.2`
- Priority: `P0`
- Expected ETA: `same-day required verification`

## Objective
- 目标描述：验证 execution bridge 绑定 `StorageProfileConfig` 后，真实 `release_default` 样本的 checkpoint cadence 与 status `effective_budget` 一致。
- 成功标准：`<64` 不提前出现 checkpoint，`height=64` 生成首个 checkpoint，storage gate 与 evidence 文档完成回写。
- 非目标：不在本轮扩展 soak / GC fail-safe / launcher 联调范围。

## Current State
- 当前实现 / 文档状态：已锁定根因为 execution bridge 仍使用硬编码 `32/4`，未实际消费 `StorageProfileConfig`。
- 已确认事实：真实 probe 在 `height=32` 已观察到 `checkpoint_count=1`；公开 API 不存在显式 `save/flush/checkpoint` 路径；`feedback_submit` 有 `10/60s` 限流。
- 待确认假设：修复后 `release_default` 的首个 checkpoint 将与 `effective_budget.execution_checkpoint_interval=64` 对齐。
- 当前失败信号 / 用户反馈：T7.2 gate/status 预算口径不一致，会误导发布门禁判断。

## Scope
- In Scope:
  - 复跑真实 `world_chain_runtime --storage-profile release_default` 样本。
  - 复跑 `scripts/world-runtime-storage-gate.sh` 并确认 checkpoint 生成高度。
  - 回写测试记录、失败签名或放行结论。
- Out of Scope:
  - 修改 runtime 实现逻辑。
  - 扩展到 transfer/explorer/launcher 其他功能验收。

## Inputs
- 关键文件：`crates/agent_world/src/bin/world_chain_runtime/execution_bridge.rs`、`crates/agent_world/src/bin/world_chain_runtime.rs`、`doc/world-runtime/evidence/runtime-storage-gate-sample-2026-03-10.md`
- 关键命令：`env -u RUSTC_WRAPPER cargo test -p agent_world node_runtime_execution_driver_uses_storage_profile_checkpoint_interval -- --nocapture`、真实 runtime probe + gate 复跑命令
- 上游依赖：`runtime_engineer` 已完成 profile 透传修复
- 现有测试 / 证据：`doc/world-runtime/evidence/runtime-storage-gate-sample-2026-03-10.md`、`.tmp/runtime_t72_probe_20260310-235637/artifacts/*`

## Requested Work
- 工作项 1：验证定向单测通过并锁定 `release_default` 在 `64` 才生成 checkpoint。
- 工作项 2：复跑真实 runtime 样本，确认 `<64` 无 checkpoint、`>=64` 有 checkpoint。
- 工作项 3：回写 `doc/world-runtime/evidence/*`、`doc/world-runtime/project.md`、`doc/devlog/2026-03-10.md`。

## Expected Outputs
- 代码改动：无
- 文档回写：T7.2 evidence / project / devlog
- 测试记录：`test_tier_required`
- devlog 记录：必填

## Done Definition
- [ ] 输出满足目标与成功标准
- [ ] 影响面已核对上游 / 下游角色
- [ ] 对应 `prd.md` / `project.md` 已回写
- [ ] 对应 `doc/devlog/YYYY-MM-DD.md` 已记录
- [ ] required/full 测试证据已补齐

## Risks / Decisions
- 已知风险：真实 probe 受 feedback rate limit 影响，可能需要更长时间窗口或改用其他 commit 驱动路径。
- 待拍板事项：若真实样本仍与单测不一致，需重新升级为 runtime P0。
- 建议决策：先以定向单测锁住 profile 透传，再做真实样本 required 复核。

## Validation Plan
- 测试层级：`test_tier_required`
- 验证命令：定向 cargo test + `world_chain_runtime` 真实 probe + `scripts/world-runtime-storage-gate.sh`
- 预期结果：status `effective_budget` 与实际 checkpoint cadence 一致
- 回归影响范围：`world-runtime` storage gate / replay evidence / release gate judgement

## Handoff Acknowledgement
- 接收方确认范围：`待 qa_engineer 回填`
- 接收方确认 ETA：`待 qa_engineer 回填`
- 接收方新增风险：`待 qa_engineer 回填`
