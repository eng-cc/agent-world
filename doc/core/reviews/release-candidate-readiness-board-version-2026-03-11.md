# 发布候选 readiness 看板（Version Candidate / 2026-03-11）

审计轮次: 4

## Meta
- Candidate ID: `VERSION-CANDIDATE-20260311-A`
- Date: `2026-03-11`
- Base Candidate: `CANDIDATE-GAME-018-ROUND009`
- Owner Role: `producer_system_designer`
- Review Partner: `qa_engineer`
- Overall Status: `conditional`

## Inherited Ready Slots
| Slot | Source | Status | Evidence Path | Note |
| --- | --- | --- | --- | --- |
| `gameplay` | `CANDIDATE-GAME-018-ROUND009` | `ready` | `doc/game/gameplay/gameplay-micro-loop-visual-closure-evidence-2026-03-10-round009.md` | 作为版本级候选的 gameplay 基线 |
| `playability` | `CANDIDATE-GAME-018-ROUND009` | `ready` | `doc/playability_test_result/evidence/playability-release-evidence-bundle-task-game-018-2026-03-10.md` | task 级评分与结论可继承 |
| `testing` | `CANDIDATE-GAME-018-ROUND009` | `ready` | `doc/testing/evidence/release-evidence-bundle-task-game-018-2026-03-10.md` | 统一测试证据包已存在 |
| `runtime_base` | `CANDIDATE-GAME-018-ROUND009` | `ready` | `doc/world-runtime/evidence/runtime-release-gate-metrics-task-game-018-2026-03-10.md` | 仅代表 task 级 runtime 边界验收 |
| `core` | `CANDIDATE-GAME-018-ROUND009` | `ready` | `doc/core/reviews/stage-closure-go-no-go-task-game-018-2026-03-10.md` | core task 级汇总结论已存在 |

## Version Runtime Extension Slots
| Slot | Owner | Status | Evidence Path | Blocker | Next Action |
| --- | --- | --- | --- | --- | --- |
| `runtime_footprint` | `runtime_engineer` | `watch` | `doc/world-runtime/runtime/runtime-storage-footprint-governance-2026-03-08.prd.md` | 当前只有治理专题与目标态，无候选级实测样本摘要 | 产出版本级 footprint 实测记录 |
| `runtime_gc` | `runtime_engineer` | `watch` | `doc/world-runtime/runtime/runtime-storage-footprint-governance-2026-03-08.prd.md` | 当前只有 GC/retention 目标与契约，无候选级结果归档 | 产出候选级 GC 结果与恢复摘要 |
| `runtime_soak` | `runtime_engineer` / `qa_engineer` | `blocked` | `doc/world-runtime/runtime-p0-candidate-evidence-handoff-2026-03-10.md` / `doc/testing/longrun/s10-five-node-real-game-soak.prd.md` / `doc/game/gameplay/gameplay-longrun-p0-production-hardening-2026-03-06.prd.md` | 尚未把版本级 soak / longrun / operability 联合证据绑定到当前候选 | 在下一任务中补齐版本级联合证据记录 |

## Aggregation Rule
- Inherited ready 项只代表“task 级已确认”，不足以单独构成版本级 ready。
- `runtime_footprint`、`runtime_gc`、`runtime_soak` 任一 `blocked`：版本级总状态不得高于 `conditional`。
- 仅当新增三槽位全部 `ready` 时，版本级候选才可进入 `ready` 评审。

## Current Decision
- Current Decision: `conditional`
- Reason:
  - task 级已 ready 的 gameplay / playability / testing / runtime_base / core 可直接继承。
  - 版本级新增槽位中 `runtime_soak` 仍为 `blocked`，`runtime_footprint` / `runtime_gc` 仍为 `watch`。
  - 因此当前版本级候选已具备结构化入口，但仍未达到最终 release ready。

## Recommended Follow-Up
- 第一跟进项：执行 `TASK-CORE-020`，由 `runtime_engineer` / `qa_engineer` 绑定版本级 footprint / GC / soak 联合证据。
- 第二跟进项：在同一版本级 board 中把 `watch`/`blocked` 三槽位刷新为真实结论。
- 第三跟进项：保留 `CANDIDATE-GAME-018-ROUND009` 作为 task 基线，不直接修改原 task board。
