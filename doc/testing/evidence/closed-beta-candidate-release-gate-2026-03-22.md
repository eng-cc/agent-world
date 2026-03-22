# Unified Closed-Beta Candidate Release Gate Evidence (2026-03-22)

审计轮次: 3

## Meta
- Gate ID: `GATE-RESET-20260322-CLOSEDBETA`
- Subject: `closed_beta_candidate` release gate that must run on the same candidate for headed Web/UI, pure API, no-UI smoke, longrun/recovery, and QA trend baseline before any stage upgrade.
- Owner role: `qa_engineer`
- Supporting roles: `runtime_engineer` / `viewer_engineer` / `liveops_community`
- Evidence anchors:
  - `doc/playability_test_result/card_2026_03_19_09_40_56.md`
  - `doc/testing/evidence/pure-api-parity-validation-2026-03-19.md`
  - `doc/testing/evidence/post-onboarding-headless-smoke-2026-03-19.md`
  - `doc/testing/evidence/closed-beta-runtime-s10-2026-03-22.md`
  - `doc/game/gameplay/gameplay-longrun-p0-production-hardening-2026-03-06.prd.md`
  - `doc/testing/evidence/testing-quality-trend-baseline-2026-03-11.md`

## Gate Status Table
| Lane | Owner | Marker Evidence | Current Status | Next Action |
| --- | --- | --- | --- | --- |
| Headed Web/UI `#46` | `viewer_engineer` / `qa_engineer` | `doc/playability_test_result/card_2026_03_19_09_40_56.md` / `doc/testing/evidence/release-evidence-bundle-task-game-018-2026-03-10.md` | Required-tier Web run showed Mission HUD switch, `goal_id=post_onboarding.establish_first_capability`, no P0 blockers reported. | Re-run same candidate with longer recording (release gate sample) to prove broader camera angles and confirm no new high-severity log noise. |
| Pure API parity | `runtime_engineer` / `qa_engineer` | `doc/testing/evidence/pure-api-parity-validation-2026-03-19.md` | `parity_verified` on source required/full after runtime snapshot fix; canonical `player_gameplay` now wired. | Run one `closed_beta_candidate` bundle sample with no-LLM required/full plus fresh bundle longrun to prove parity persists. |
| No-UI live smoke | `liveops_community` / `qa_engineer` | `doc/testing/evidence/post-onboarding-headless-smoke-2026-03-19.md` | Headless protocol smoke confirmed `control_completion_ack=advanced`, runtime events non-empty for same session and Stage transitions. | Replay the smoke in this candidate’s bundle to capture the corresponding `snapshot`/`event` artifacts and link to gate report. |
| Longrun & recovery | `runtime_engineer` | `doc/testing/longrun/s10-five-node-real-game-soak.prd.md` / `doc/game/gameplay/gameplay-longrun-p0-production-hardening-2026-03-06.prd.md` / `doc/testing/evidence/closed-beta-runtime-s10-2026-03-22.md` | `block`。cleanup 修复后，clean-room `120s` 候选样本 `output/longrun/closed-beta-rerun-fixed-20260322/20260322-120458` 已 `process_status=ok / metric_gate=pass`；但 runtime lane 仍缺同一 candidate 的 `600s+` soak 与 replay/rollback drill，所以还不能转成放行证据。 | 沿用 clean-room 运行条件补齐 `600s+` soak、replay/rollback drill，并把最终 `pass/block` 结果回填本门禁文档。 |
| Trend baseline | `qa_engineer` | `doc/testing/evidence/testing-quality-trend-baseline-2026-03-11.md` | Latest baseline has first-pass 33.3%, escape 66.7%, fix time 0.33d. | Add at least two new representative samples (make them `pass` or `pass_after_fix`) so first-pass >= 60% and escape <= 20% before claiming stage upgrade. |

## QA Verdict
- 当前统一 gate 结论: `block`
- 阻断原因:
  - Longrun & recovery lane 已拿到 clean-room `120s` `pass` 证据，但同一 candidate 的 `600s+` soak 与 replay/rollback drill 仍未完成，因此 runtime lane 还不能算正式收口。
  - Trend baseline 仍低于阶段升级阈值（first-pass `< 60%`，escape `> 20%`）。
  - Headed Web/UI、pure API、no-UI smoke 还没有在同一 `closed_beta_candidate` 版本上重跑并互链，所以当前只能算“lane 准备中”，不能算统一 gate 已通过。
- 允许的结论:
  - 可以提交并维护本 gate 文档，作为 `TASK-GAME-031` 的 QA 汇总入口。
  - 不可以把当前 gate 文档当成 `closed_beta_candidate approved` 或 `TASK-GAME-033 go` 证据。

## Gate Execution Notes
- Candidate definition: use the fresh bundle that passes `TASK-GAME-018` evidence (see `doc/testing/evidence/release-evidence-bundle-task-game-018-2026-03-10.md`) plus pure API parity smoke artifacts; reference this candidate in all lane logs.
- Gate rule: every lane must run on the same candidate tag/version/date; mixing old evidence is not allowed. Log command + stdout path for each lane (build the evidence bundle folder under `output/playwright/playability/closed-beta-...`).
- Blocking conditions:
  - Any `blocking` failure from the gate (e.g., longrun soak timed out, headed Web noise persists, parity regression) immediately keeps stage at `internal_playable_alpha_late`.
  - Trend baseline metrics below thresholds prevent upgrading regardless of other lane statuses.
  - Runtime 侧的短诊断样本不能替代 release-gate evidence；当前唯一可作为候选真值的 runtime 结果是 clean-room `120s` pass，但它仍不足以替代 `600s+` soak + replay/rollback。

## Next Steps
1. 保持本 gate 文档的总体结论为 `block`，直到 runtime lane 提供同一 candidate 下的 clean-room `600s+` soak + replay/rollback evidence。
2. 让 headed Web/UI、pure API、no-UI smoke 在同一 candidate 上补跑 fresh sample，并把命令、stdout/stderr 路径、证据 bundle 目录回填到本表。
3. 补齐至少两条新的 trend baseline 样本，把 QA 趋势指标推到 first-pass `>= 60%`、escape `<= 20%` 后，再交 `producer_system_designer` 做 `TASK-GAME-033` 阶段评审。

## Outstanding Inputs
- Candidate build/fresh bundle from `runtime_engineer` / `viewer_engineer`.
- Runtime clean-room `600s+` soak 与 replay/rollback evidence on the same candidate.
- Additional `trend baseline` samples (pass/escape timing) from ongoing QA runs.
- Confirmation from `liveops_community` that no new high-visibility communication (e.g., “closed beta” or “play now”) leaked before the gate passes.
