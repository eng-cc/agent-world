# Viewer 首局目标清晰度加固（2026-02-27）项目管理文档

- 对应设计文档: `doc/world-simulator/viewer/viewer-first-session-goal-clarity-hardening-2026-02-27.design.md`
- 对应需求文档: `doc/world-simulator/viewer/viewer-first-session-goal-clarity-hardening-2026-02-27.prd.md`

审计轮次: 5
## 任务拆解（含 PRD-ID 映射）
- [x] T0 建档：设计文档 + 项目管理文档
- [x] T1 主任务结构化：动作句 + 完成条件 + 预计耗时 + 次任务折叠
- [x] T2 下一步按钮与主任务剩余提示
- [x] T3 卡住检测（5 秒无进展）与恢复提示
- [x] T4 首局结算卡片 + 测试 + 文档收口
- [x] T5 (PRD-VIEWER-FSGC-005) [test_tier_required]: 修复隐藏态下 stuck 提示与 onboarding 引导卡重叠，补齐锚点分支测试并回写文档/devlog。
- [x] T6 (PRD-VIEWER-FSGC-006) [test_tier_required]: 在 `4/4` 完成后切入 `PostOnboarding` 阶段目标卡，补齐默认目标 / 阻塞解释 / 分支解锁与 summary 文案更新，并完成定向回归。

## 依赖
- doc/world-simulator/viewer/viewer-first-session-goal-clarity-hardening-2026-02-27.prd.md
- `doc/world-simulator/viewer/viewer-first-session-goal-control-feedback-2026-02-27.prd.md`
- `doc/playability_test_result/game-test.prd.md`
- `crates/agent_world_viewer/src/egui_right_panel_player_guide.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_experience.rs`

## 状态
- 最近更新：2026-03-18（T6 完成：`4/4` 后切入 `PostOnboarding` 阶段目标卡）
- 当前阶段：已完成（T0~T6）
- 阻塞项：无
