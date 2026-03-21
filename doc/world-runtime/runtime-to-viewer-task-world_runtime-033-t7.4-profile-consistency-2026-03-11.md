# Role Handoff Detailed

## Meta
- Handoff ID: `HANDOFF-WORLD_RUNTIME-033-T7.4-2026-03-11-PROFILE-CONSISTENCY`
- Date: `2026-03-11`
- From Role: `runtime_engineer`
- To Role: `viewer_engineer`
- Related Module: `world-runtime`
- Related PRD-ID: `PRD-WORLD_RUNTIME-014/015`
- Related Task ID: `TASK-WORLD_RUNTIME-033 / T7.4`
- Priority: `P1`
- Expected ETA: `same-day follow-up`

## Objective
- 目标描述：在已锁住 CLI 参数透传后，继续完成 launcher / bundle / Web 侧对三档 storage profile 的一致性验证。
- 成功标准：bundle 入口、launcher 参数与最终 `oasis7_chain_runtime --storage-profile` 口径一致，并沉淀正式证据。
- 非目标：不在本轮重做 runtime 内部 retention/GC 逻辑。

## Current State
- 当前实现 / 文档状态：`runtime_engineer` 已新增 game/web launcher 的三档 profile 参数透传回归。
- 已确认事实：`oasis7_chain_runtime`、`oasis7_game_launcher`、`oasis7_web_launcher` 都支持 `dev_local/release_default/soak_forensics`；bundle wrapper 通过 `OASIS7_CHAIN_STORAGE_PROFILE` 覆盖。
- 待确认假设：bundle / launcher 的实际运行产物与 CLI/单测口径一致。
- 当前失败信号 / 用户反馈：暂无新增失败；当前缺的是 T7.4 的正式跨入口证据。

## Scope
- In Scope:
  - bundle wrapper 与 launcher 的 profile 透传实测/证据。
  - Web / game launcher 对 `chain-storage-profile` 的跨入口一致性回写。
- Out of Scope:
  - 修改 runtime retention 行为。
  - Soak 长跑本体。

## Inputs
- 关键文件：`crates/oasis7/src/bin/oasis7_game_launcher/oasis7_game_launcher_tests.rs`、`crates/oasis7/src/bin/oasis7_web_launcher/oasis7_web_launcher_tests.rs`、`scripts/build-game-launcher-bundle.sh`
- 现有测试 / 证据：新增 tri-profile 参数透传回归；既有 T6.4/T6.5 文档记录

## Requested Work
- 工作项 1：补齐 bundle/launcher 实测证据。
- 工作项 2：回写 T7.4 evidence / project / devlog。
- 工作项 3：若需要，补充脚本或 launcher 侧缺失回归。

## Expected Outputs
- 代码改动：可选，限定 launcher / bundle / docs
- 文档回写：T7.4 evidence、project、devlog
- 测试记录：`test_tier_full` 优先
- devlog 记录：必填

## Done Definition
- [x] 三档 profile 跨 launcher / bundle / runtime 口径一致
- [x] 正式 evidence 已生成
- [x] `project.md` / `devlog` 已回写
- [x] 回归证据已补齐

## Risks / Decisions
- 已知风险：若 bundle wrapper 与二进制默认值漂移，可能只会在实测中暴露。
- 待拍板事项：是否需要把 wrapper 实测纳入长期 required gate。
- 建议决策：先沉淀一次正式实测证据，再决定是否固化到脚本门禁。

## Validation Plan
- 测试层级：`test_tier_full`
- 验证命令：由 `viewer_engineer` 选择 bundle / launcher 实测命令并回写
- 预期结果：三档 profile 从 launcher/bundle 到 runtime 的参数一致
- 回归影响范围：launcher / bundle / runtime profile governance

## Handoff Acknowledgement
- 接收方确认范围：`已完成 bundle wrapper 静态/动态透传核对，并回写 T7.4 正式 evidence`
- 接收方确认 ETA：`2026-03-11 01:05 CST 已完成`
- 接收方新增风险：`当前 bundle 实测已覆盖 wrapper 参数注入；若后续要扩到完整浏览器闭环，可在 T7.5/testing-manual 再补 Web 操作步骤`
