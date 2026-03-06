# 客户端启动器优雅退出与级联进程关闭（2026-03-02）项目管理

审计轮次: 4
## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-001)：建档（设计文档 + 项目管理文档）。
- [x] T1 (PRD-WORLD_SIMULATOR-002)：改造停止流程（优雅中断 + 超时强杀兜底，覆盖按钮停止与窗口关闭）。
- [x] T2 (PRD-WORLD_SIMULATOR-003)：补充测试并处理单文件行数约束。
- [x] T3 (PRD-WORLD_SIMULATOR-003)：回归测试、文档收口与任务日志更新。

## 依赖
- doc/world-simulator/launcher/game-client-launcher-graceful-stop-2026-03-02.prd.md
- `crates/agent_world_client_launcher` 现有进程托管逻辑。
- `crates/agent_world/src/bin/world_game_launcher.rs` 的 signal 退出处理。
- `testing-manual.md` 测试分层规范。

## 状态
- 当前阶段：已完成（T0~T3）。
- 当前任务：无（项目结项）。
