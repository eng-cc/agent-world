# Viewer 控制反馈 P0：Step 卡住恢复与强反馈（2026-02-27）项目管理文档

## 任务拆解
- [x] T0 建立设计文档与项目管理文档，明确 P0 边界与验收口径
- [ ] T1 实现 `step` 卡住自动恢复（`seek(tick+1)`）+ Cause/Next 强反馈文案
- [ ] T2 执行定向测试与 A/B 实测，回写文档与日志并收口

## 依赖
- `doc/playability_test_result/card_2026_02_27_20_32_17.md`
- `doc/world-simulator/viewer-control-feedback-iteration-checklist-2026-02-27.md`
- `crates/agent_world_viewer/src/web_test_api.rs`
- `scripts/run-game-test-ab.sh`

## 状态
- 当前阶段：进行中（下一步 T1）
- 阻塞项：无
