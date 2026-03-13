# OpenClaw 与内置 Agent 体验等价（parity）验收方案（2026-03-12）项目管理文档

- 对应设计文档: `doc/world-simulator/llm/llm-openclaw-agent-experience-parity-2026-03-12.design.md`
- 对应需求文档: `doc/world-simulator/llm/llm-openclaw-agent-experience-parity-2026-03-12.prd.md`

审计轮次: 1

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-038) [test_tier_required]: 完成 `OpenClaw` 与内置 agent 体验等价（parity）专题 PRD / Design / Project 建模，并回写模块主文档、索引与 devlog。
- [x] T1 (PRD-WORLD_SIMULATOR-038) [test_tier_required]: 冻结 `P0/P1/P2` 场景集、评分项、通过线与阻断线，并新增 parity 场景矩阵与评分卡模板。
- [x] T2 (PRD-WORLD_SIMULATOR-038) [test_tier_required]: 为 builtin 与 OpenClaw provider 冻结统一 fixture benchmark 协议、trace 汇总字段与分数聚合模板。
- [x] T3 (PRD-WORLD_SIMULATOR-038) [test_tier_required]: 将 `OpenClaw(Local HTTP)` 专题与 `Decision Provider` 专题的实施任务改挂到 parity 目标，确保“接通”不等于“完成”。
- [ ] T4 (PRD-WORLD_SIMULATOR-038) [test_tier_full]: 完成真实 `OpenClaw(Local HTTP)` 的 `P0` parity 对标试玩，输出 QA/producer 双签结论。
- [ ] T5 (PRD-WORLD_SIMULATOR-038) [test_tier_full]: 在 `P0` 通过后推进 `P1`/`P2`，并依据结果决定是否允许默认启用或扩大覆盖范围。

## 依赖
- `doc/world-simulator/llm/llm-decision-provider-standard-openclaw-feasibility-2026-03-12.prd.md`
- `doc/world-simulator/llm/llm-openclaw-local-http-provider-integration-2026-03-12.prd.md`
- `doc/world-simulator/prd/acceptance/openclaw-agent-parity-scenario-matrix-2026-03-12.md`
- `doc/world-simulator/prd/acceptance/openclaw-agent-parity-score-card-2026-03-12.md`
- `doc/world-simulator/prd/acceptance/openclaw-agent-parity-benchmark-protocol-2026-03-12.md`
- `doc/world-simulator/prd/acceptance/openclaw-agent-parity-aggregation-template-2026-03-12.md`
- `doc/world-simulator/prd.md`
- `doc/world-simulator/project.md`
- `doc/world-simulator/prd.index.md`
- `doc/world-simulator/llm/openclaw-agent-profile-agent_world_p0_low_freq_npc-2026-03-13.md`

## 状态
- 最近更新：2026-03-13
- 当前阶段: T4 in_progress
- 当前任务: `使用 scripts/openclaw-parity-p0.sh 采集真实 OpenClaw(Local HTTP) 的 P0 parity 证据，并补齐 QA/producer 双签材料`
- owner: `agent_engineer`
- 联审: `qa_engineer`、`viewer_engineer`、`runtime_engineer`
- 发起建模: `producer_system_designer`
- 备注: 本专题将“体验等价”提升为上线门禁；后续若 `OpenClaw` 未达到 parity，只允许保留在 `experimental`，不得标记为默认体验。

- T4 进展备注: 已落地 `crates/agent_world/src/bin/world_openclaw_parity_bench.rs` 与 `scripts/openclaw-parity-p0.sh`，用于按 `PRD-WORLD_SIMULATOR-038` benchmark 协议输出 `raw/*.jsonl`、单样本 summary、聚合 `combined.csv`、`failures.md` 与 `scorecard-links.md`；已通过 `openclaw_local_http` mock localhost smoke 验证产物结构，真实 builtin/OpenClaw 对标仍待本机 provider 与 QA/producer 实测。
- T4 口径补充: parity harness 已新增 `--openclaw-agent-profile` 并默认固定到 `agent_world_p0_low_freq_npc`；`DecisionRequest.agent_profile`、summary provider 信息与批处理脚本现已保留该 profile，便于 QA/producer 确认样本不是在“未知通用 skill”下跑出来的结果。
