# Viewer 控制可预期性改版任务清单（2026-02-28）项目管理文档

## 任务拆解
- [x] T1 读取最新卡片与近 6 轮量化结果，提炼问题基线
- [x] T2 输出设计文档（目标/范围/接口数据/里程碑/风险）
- [x] T3 形成可执行任务清单（P0/P1/P2，含交付与验收）
- [x] T4 形成 A/B 实验矩阵与止损规则
- [x] T5 文档收口（更新 devlog 并提交）
- [x] T6 落地 P0.1：Web Test API 阶段语义改为 `received/executing/completed_advanced/completed_no_progress/blocked`
- [x] T7 落地 P0.1：Mission HUD 控制反馈卡片四态展示（Completed 合并展示）
- [x] T8 落地 P0.1：补齐控件阶段标签映射测试（含 legacy `applied` 兼容）
- [x] T9 执行 test_tier_required 定向验证（viewer 单测 + wasm32 check）
- [x] T10 文档收口（更新项目状态与 devlog，提交本任务）
- [x] T11 落地 P0.2：移除 `step` 卡住后的自动 `play` 代跑，改为 `completed_no_progress` 终态
- [x] T12 落地 P0.2：Mission HUD 控制反馈卡片增加无进展恢复 CTA（Recover play / Retry step x8）
- [x] T13 落地 P0.2：避免 stuck hint 与 control feedback 卡片重复渲染恢复按钮
- [x] T14 落地 P0.2：补齐恢复 CTA 判定单测
- [x] T15 执行 test_tier_required 定向验证（viewer 单测 + wasm32 check）并文档收口
- [x] T16 落地 P0.3：执行 10 轮 A/B 回归（沿用 `scripts/run-game-test-ab.sh`，以 headed 模式完成）
- [x] T17 落地 P0.3：汇总 10 轮指标并做门槛判定（`B>=80%`、`maxNoProgressWindow<=2500ms`）
- [x] T18.1 进入 P0.4：三轮定向复测并定位 `play/step` 无进展根因（共识链路）
- [x] T18.2a 进入 P0.4：A/B 脚本补齐失败原因分类（`failCategory` + `feedbackStage/Reason/Hint`）
- [ ] T18.2b 进入 P0.4：修复 `play` 持续推进与 `step` followup 一致性，收敛 B 段与 stall 指标

## 依赖
- `doc/playability_test_result/card_2026_02_28_19_22_20.md`
- `doc/playability_test_result/card_2026_02_27_23_21_00.md`
- `doc/playability_test_result/card_2026_02_27_21_04_46.md`
- `doc/world-simulator/viewer-step-completion-ack-2026-02-28.md`
- `doc/world-simulator/viewer-control-feedback-iteration-checklist-2026-02-27.md`
- `scripts/run-game-test-ab.sh`
- `output/playwright/playability/p0_3_20260228-201225/p0_3_summary.json`
- `output/playwright/playability/p0_3_20260228-201225/p0_3_summary.md`
- `output/playwright/playability/20260228-202833/ab_metrics.json`
- `output/playwright/playability/20260228-203202/ab_metrics.json`
- `output/playwright/playability/20260228-203526/ab_metrics.json`
- `output/playwright/playability/20260228-204632/ab_metrics.json`
- `crates/agent_world_viewer/src/web_test_api.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_guide.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_mission_tests.rs`

## 状态
- 当前阶段：进行中（T1~T18.2a 已完成，执行 T18.2b）
- 当前结论：
  - 已形成一份可直接落地的“控制可预期性改版”执行清单，覆盖 P0/P1/P2、量化门槛与 A/B 实验项。
  - P0.1 已落地：控制反馈阶段语义与 UI 展示完成对齐，`completed_no_progress` 不再误标为 `blocked`。
  - P0.2 已落地：无进展恢复 CTA 已进入玩家可操作闭环（不再自动代跑），玩家可显式触发 `play/step` 恢复动作。
  - P0.3 回归（`output/playwright/playability/p0_3_20260228-201225`）结果：`B pass rate=70% (7/10)`、`stall>2500ms=100% (10/10)`、`max stall=5892ms`，未达到发布门槛。
  - P0.4 诊断（`20260228-202833`/`20260228-203202`/`20260228-203526`）显示：当前共识链路下 `play` 与 `step followup` 仍频繁“accepted 但无推进”，单轮最差 `effectiveControlHitRate=0%`、`maxNoProgressWindowMs≈6s`。
  - 已做一次 runtime 试验性改动用于验证假设，但复测未证实收益，改动已回滚，未进入主线。
  - T18.2a 已完成：`scripts/run-game-test-ab.sh` 现已输出 `failCategory` 与 `feedbackStage/feedbackReason/feedbackHint`，B 段 FAIL 不再是“未知失败”；样本 `20260228-204632` 已验证字段落盘。
  - 下一优先级：执行 T18.2b，围绕共识链路的 `play` 持续驱动与 `step` 后续推进语义做针对性修复，再做 10 轮门槛回归。
- 阻塞项：
  - headless Web 路径在当前环境存在 WebGL 上下文丢失（`CONTEXT_LOST_WEBGL`），P0.3 已改为 headed 执行；后续若恢复 headless 需先修复图形链路稳定性。
- 最近更新：2026-02-28
