# Runtime 版本级候选联合证据（2026-03-11）

审计轮次: 4

## Meta
- Evidence ID: `RT-VERSION-CANDIDATE-20260311`
- Date: `2026-03-11`
- Owner Role: `runtime_engineer`
- Scope: `version candidate runtime footprint / GC / soak`
- Conclusion: `partial_ready`

## Slot Summary
| Slot | Status | Evidence Path | Conclusion |
| --- | --- | --- | --- |
| `runtime_footprint` | `ready` | `doc/world-runtime/evidence/runtime-storage-gate-sample-2026-03-10.md` | 已有真实 `release_default` 样本与 gate 摘要，可用于版本级 footprint 槽位。 |
| `runtime_gc` | `ready` | `doc/world-runtime/evidence/runtime-sidecar-orphan-gc-failsafe-2026-03-11.md` | 已证明 sidecar orphan 为窗口态且可在后续 save/GC 后收敛。 |
| `runtime_soak` | `blocked` | `doc/world-runtime/runtime-p0-candidate-evidence-handoff-2026-03-10.md` / `doc/testing/longrun/s10-five-node-real-game-soak.prd.md` | 当前只有专题与 handoff，仍缺真实版本级 soak summary / metrics 绑定。 |

## Footprint Evidence
- 证据入口：`doc/world-runtime/evidence/runtime-storage-gate-sample-2026-03-10.md`
- 可采信结论：
  - `release_default` profile 下真实 runtime 样本已通过 storage gate 接线。
  - QA 复验已确认 `<64` 不提前出现 checkpoint、`65` 时出现首个 checkpoint。
  - 该证据足以把版本级 `runtime_footprint` 从 `watch` 提升到 `ready`。

## GC Evidence
- 证据入口：`doc/world-runtime/evidence/runtime-sidecar-orphan-gc-failsafe-2026-03-11.md`
- 可采信结论：
  - sidecar orphan 并非稳定泄漏，而是 save/GC 时序窗口信号。
  - 自动化测试已证明下一次成功 `save_to_dir()` 后 `orphan_blob_count` 可收敛到 `0`。
  - 该证据足以把版本级 `runtime_gc` 从 `watch` 提升到 `ready`。

## Soak Gap
- 当前缺口：
  - 现有 task 级 runtime release gate 指标记录中已明确 `.tmp/s10_longrun_t2/t3` 摘要仍为 `dry_run`。
  - `runtime_p0_candidate_evidence_handoff` 也指出版本级仍需真实 soak / longrun / operability 联合证据。
- 结论：
  - `runtime_soak` 仍必须保持 `blocked`，直到有真实版本级 soak summary / metrics 文档绑定到当前候选。

## Overall Interpretation
- runtime 在版本级候选上已从“只有 task 级边界验收”提升到“footprint + GC 有真实可引用证据，soak 仍缺”。
- 因此 runtime 联合证据的当前结论应为 `partial_ready`，而非整体 `ready`。
