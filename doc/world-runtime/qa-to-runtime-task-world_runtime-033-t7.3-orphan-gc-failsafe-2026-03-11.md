# Role Handoff Detailed

## Meta
- Handoff ID: `HANDOFF-WORLD_RUNTIME-033-T7.3-2026-03-11-ORPHAN-GC`
- Date: `2026-03-11`
- From Role: `qa_engineer`
- To Role: `runtime_engineer`
- Related Module: `world-runtime`
- Related PRD-ID: `PRD-WORLD_RUNTIME-014/015`
- Related Task ID: `TASK-WORLD_RUNTIME-033 / T7.3`
- Priority: `P1`
- Expected ETA: `same-day follow-up`

## Objective
- 目标描述：把 T7.2 QA 复验里暴露的 pre-checkpoint 窗口瞬时 `orphan_blob_count=1` 收敛为可解释、可回归、可放行的 GC fail-safe 证据。
- 成功标准：明确该 orphan 是预期短暂窗口还是异常泄漏；补齐自动化或脚本化证据，证明后续 GC/checkpoint 会把 orphan 收敛到 `0`。
- 非目标：不在本轮扩展 profile 切换、archive read、checkpoint corruption、replay mismatch 全量套件。

## Current State
- 当前实现 / 文档状态：T7.2 已完成；真实 `release_default` 样本在 `height=47` 时 `checkpoint_count=0 / orphan_blob_count=1 / full_log_only`，在 `height=65` 时收敛到 `checkpoint_count=1 / orphan_blob_count=0 / checkpoint_plus_log`。
- 已确认事实：profile cadence 修复已生效，当前唯一新增信号是 pre-checkpoint 窗口出现过瞬时 orphan。
- 待确认假设：该 orphan 来自 checkpoint 前的临时保留/GC 时序窗口，而非 CAS 引用泄漏。
- 当前失败信号 / 用户反馈：storage gate 在 pre64 样本上除 `checkpoint_count=0` 外，还出现 `orphan_blob_count=1`。

## Scope
- In Scope:
  - 复现并解释 pre-checkpoint orphan 窗口。
  - 增加定向回归或脚本化证据，证明 orphan 最终收敛到 `0`。
  - 回写 runtime evidence / project / devlog。
- Out of Scope:
  - Soak profile 联调。
  - Launcher/UI 侧展示改动。

## Inputs
- 关键文件：`doc/world-runtime/evidence/runtime-storage-gate-sample-2026-03-10.md`、`crates/oasis7/src/bin/world_chain_runtime/execution_bridge.rs`、`crates/oasis7/src/bin/world_chain_runtime/storage_metrics.rs`
- 关键样本：`.tmp/runtime_t72_qa_posfast_20260311-002402/artifacts/status-pre64.json`、`.tmp/runtime_t72_qa_posfast_20260311-002402/artifacts/status-post64.json`
- 现有测试 / 证据：`node_runtime_execution_driver_uses_storage_profile_checkpoint_interval`、T7.2 QA gate 摘要

## Requested Work
- 工作项 1：定位 orphan 在 pre-checkpoint 窗口出现的具体引用/GC 原因。
- 工作项 2：补一条定向回归，锁住“最终 orphan 归零”的 fail-safe 语义。
- 工作项 3：回写 T7.3 evidence 与 project 状态。

## Expected Outputs
- 代码改动：如需，限定在 runtime storage/metrics/bridge 范围
- 文档回写：`doc/world-runtime/evidence/*`、`doc/world-runtime/project.md`、专题 `project.md`
- 测试记录：`test_tier_full` 优先，必要时附 required 定向命令
- devlog 记录：必填

## Done Definition
- [x] orphan 窗口来源已解释清楚
- [x] 收敛到 `0` 的证据可重复
- [x] `project.md` / `devlog` 已回写
- [x] 测试证据已补齐

## Risks / Decisions
- 已知风险：若 orphan 不是短暂窗口而是实际泄漏，T7.3 可能升级为实现修复任务。
- 待拍板事项：是否需要把 pre64 orphan 容忍策略编码进 gate，或坚持只接受最终稳定采样。
- 建议决策：先证明其为短暂窗口，再决定是否调整 gate 采样策略。

## Validation Plan
- 测试层级：`test_tier_full`
- 验证命令：待 `runtime_engineer` 根据定位结果补齐
- 预期结果：pre-checkpoint 窗口可解释，post-checkpoint/GC 稳态 orphan 为 `0`
- 回归影响范围：runtime storage metrics / release gate / QA sampling policy

## Handoff Acknowledgement
- 接收方确认范围：`已完成 orphan / GC fail-safe 解释、定向回归与文档回写`
- 接收方确认 ETA：`2026-03-11 00:45 CST 已完成`
- 接收方新增风险：`若后续要把该窗口直接编码进 gate 采样策略，仍需在 T7.4/T7.5 进一步统一 QA 取样口径`
