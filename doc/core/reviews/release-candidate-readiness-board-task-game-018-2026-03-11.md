# 发布候选 readiness 看板（TASK-GAME-018 / 2026-03-11）

审计轮次: 4

## Meta
- Candidate ID: `CANDIDATE-GAME-018-ROUND009`
- Date: `2026-03-11`
- Scope: `TASK-GAME-018 / ROUND-009`
- Owner Role: `producer_system_designer`
- Review Partner: `qa_engineer`
- Overall Status: `conditional`

## Candidate Header
- Candidate Summary: 基于 `TASK-GAME-018` 的 gameplay 视觉优化收口、playability/testing 证据包与 runtime 边界验收，当前已具备任务级 `conditional-go` 所需最小闭环。
- Source Go/No-Go: `doc/core/reviews/stage-closure-go-no-go-task-game-018-2026-03-10.md`
- Next Escalation Goal: 从 task 级候选扩展到更高层 release candidate，补齐 runtime footprint / GC / soak 联合验证。

## P0 Evidence Slots
| Slot | Owner | Status | Evidence Path | Blocker | Next Action |
| --- | --- | --- | --- | --- | --- |
| `gameplay` | `viewer_engineer` / `qa_engineer` | `ready` | `doc/game/gameplay/gameplay-micro-loop-visual-closure-evidence-2026-03-10-round009.md` | 无 | 作为后续更大候选的 gameplay 基线继续复用 |
| `playability` | `qa_engineer` | `ready` | `doc/playability_test_result/evidence/playability-release-evidence-bundle-task-game-018-2026-03-10.md` | 无 | 若升级到版本级候选，补更长录屏抽样 |
| `testing` | `qa_engineer` | `ready` | `doc/testing/evidence/release-evidence-bundle-task-game-018-2026-03-10.md` | 无 | 延续统一测试证据包格式 |
| `runtime` | `runtime_engineer` | `ready` | `doc/world-runtime/evidence/runtime-release-gate-metrics-task-game-018-2026-03-10.md` | 当前 task 级无阻断；版本级仍缺 footprint / GC / soak 联合验证 | 在下一任务中升级为候选级 runtime 长跑证据 |
| `core` | `producer_system_designer` | `ready` | `doc/core/reviews/stage-closure-go-no-go-task-game-018-2026-03-10.md` | 当前仅 task 级；尚未扩展到更大候选 | 将当前实例作为统一入口模板基线 |

## P1 Watch Slots
| Slot | Owner | Status | Evidence Path | Risk | Next Action |
| --- | --- | --- | --- | --- | --- |
| `automation_stability` | `viewer_engineer` / `qa_engineer` | `watch` | `doc/testing/evidence/release-evidence-bundle-task-game-018-2026-03-10.md` | ROUND-009 录屏样本偏短 | release gate 阶段补更长录屏抽样 |
| `headless_runtime_longrun` | `runtime_engineer` / `qa_engineer` | `watch` | `doc/headless-runtime/templates/headless-runtime-release-gate-linkage.md` | 当前非 task 级阻断，但版本级仍需挂接 | 候选升级时补入版本级入口 |

## Aggregation Rule
- P0 槽位全部 `ready`，且无 P0 `blocked`：总状态至少可达 `conditional`。
- 任一 P0 槽位 `blocked`：总状态必须为 `blocked`。
- 若所有 P0 `ready` 且 P1 仅 `watch` 无高风险阻断：可维持 `conditional`，等待升级到更大候选再决定是否 `ready`。

## Current Decision
- Current Decision: `conditional`
- Reason:
  - `TASK-GAME-018` 范围内五类 P0 槽位均已就位。
  - 当前缺口已从“任务级闭环”转移为“更大候选级 runtime footprint / GC / soak 联合验证”。
  - 因此该实例适合作为首份 readiness board 基线，但还不能代表最终版本候选已 `ready`。

## Recommended Follow-Up
- 第一跟进项：新建版本级候选扩展任务，优先补 `runtime` 槽位的 footprint / GC / soak 联合验证。
- 第二跟进项：把 `automation_stability` 的长录屏抽样纳入更大候选级 evidence pack。
- 第三跟进项：继续沿用本看板结构，不再手工拼装新的候选字段。
