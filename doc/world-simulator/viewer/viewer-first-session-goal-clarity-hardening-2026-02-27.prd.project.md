# Viewer 首局目标清晰度加固（2026-02-27）项目管理文档

审计轮次: 6
## 任务拆解（含 PRD-ID 映射）
- [x] T0 建档：设计文档 + 项目管理文档
- [x] T1 主任务结构化：动作句 + 完成条件 + 预计耗时 + 次任务折叠
- [x] T2 下一步按钮与主任务剩余提示
- [x] T3 卡住检测（5 秒无进展）与恢复提示
- [x] T4 首局结算卡片 + 测试 + 文档收口
- [x] T5 (PRD-VIEWER-FSGC-005) [test_tier_required]: 修复隐藏态下 stuck 提示与 onboarding 引导卡重叠，补齐锚点分支测试并回写文档/devlog。

## 依赖
- doc/world-simulator/viewer/viewer-first-session-goal-clarity-hardening-2026-02-27.prd.md
- `doc/world-simulator/viewer/viewer-first-session-goal-control-feedback-2026-02-27.prd.md`
- `doc/playability_test_result/game-test.prd.md`
- `crates/agent_world_viewer/src/egui_right_panel_player_guide.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_experience.rs`

## 状态
- 最近更新：2026-03-08（T5 完成：stuck 提示与 onboarding 卡防重叠修复）
- 当前阶段：已完成（T0~T5）
- 阻塞项：无
