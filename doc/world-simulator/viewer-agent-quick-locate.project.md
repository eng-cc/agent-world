# Agent World Simulator：Viewer 快速定位 Agent 按钮（项目管理文档）

## 任务拆解
- [x] QAG1：输出设计文档（`doc/world-simulator/viewer-agent-quick-locate.md`）与项目管理文档（本文件）
- [x] QAG2：新增快速定位 Agent 动作（优先当前 Agent，否则首个 Agent）
- [x] QAG3：接入按钮与多语言文案（Egui Event Link + 兼容旧 UI）
- [x] QAG4：补充测试并完成回归验证（`test_tier_required`）
- [x] QAG5：更新总项目文档与开发日志，完成任务收口

## 依赖
- `crates/agent_world_viewer/src/selection_linking.rs`
- `crates/agent_world_viewer/src/selection_linking/tests.rs`
- `crates/agent_world_viewer/src/egui_right_panel.rs`
- `crates/agent_world_viewer/src/ui_locale_text.rs`
- `doc/world-simulator.project.md`
- `doc/devlog/2026-02-15.md`

## 状态
- 当前阶段：已完成
- 最近更新：补齐旧 UI 路径系统调度（`handle_quick_locate_agent_button`）并复跑回归（2026-02-15）
