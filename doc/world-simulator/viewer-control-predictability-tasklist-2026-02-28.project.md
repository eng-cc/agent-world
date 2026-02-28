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

## 依赖
- `doc/playability_test_result/card_2026_02_28_19_22_20.md`
- `doc/playability_test_result/card_2026_02_27_23_21_00.md`
- `doc/playability_test_result/card_2026_02_27_21_04_46.md`
- `doc/world-simulator/viewer-step-completion-ack-2026-02-28.md`
- `doc/world-simulator/viewer-control-feedback-iteration-checklist-2026-02-27.md`
- `scripts/run-game-test-ab.sh`
- `crates/agent_world_viewer/src/web_test_api.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_guide.rs`
- `crates/agent_world_viewer/src/egui_right_panel_player_mission_tests.rs`

## 状态
- 当前阶段：已完成（T1~T10）
- 当前结论：
  - 已形成一份可直接落地的“控制可预期性改版”执行清单，覆盖 P0/P1/P2、量化门槛与 A/B 实验项。
  - P0.1 已落地：控制反馈阶段语义与 UI 展示完成对齐，`completed_no_progress` 不再误标为 `blocked`。
  - 当前最高优先级转为 P0.2（无进展恢复 CTA 的交互落地与回归）。
- 阻塞项：无
- 最近更新：2026-02-28
