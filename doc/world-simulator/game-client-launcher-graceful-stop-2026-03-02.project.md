# 客户端启动器优雅退出与级联进程关闭（2026-03-02）项目管理

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档。
- [x] T1 改造停止流程：优雅中断 + 超时强杀兜底，覆盖按钮停止与窗口关闭。
- [ ] T2 补充测试并处理单文件行数约束。
- [ ] T3 回归测试、文档收口与任务日志更新。

## 依赖
- `crates/agent_world_client_launcher` 现有进程托管逻辑。
- `crates/agent_world/src/bin/world_game_launcher.rs` 的 signal 退出处理。
- `testing-manual.md` 测试分层规范。

## 状态
- 当前阶段：进行中（T0~T1 已完成，执行 T2）。
- 当前任务：T2 补充测试并处理单文件行数约束。
