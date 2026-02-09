# Viewer 可视化观察体验优化（项目管理文档）

## 任务拆解
- [x] VO1：输出设计文档（`viewer-observability-visual-optimization.md`）
- [x] VO2：输出项目管理文档（本文件）
- [x] VO3：实现脚本场景预校验与别名提示（P1）
- [x] VO4：实现右侧面板观察导向布局优化（P3）
- [x] VO5：调整复制相关文案为观察导向（不做一键复制）
- [x] VO6：执行编译/测试/截图闭环验证
- [x] VO7：更新日志并提交
- [x] VO8：总览增加连接/健康/观察模式状态灯与单测
- [x] VO9：修复 3D 半屏空白并优化右侧宽度占比（边改边看）

## 依赖
- `scripts/capture-viewer-frame.sh`
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/egui_right_panel.rs`
- `crates/agent_world_viewer/src/copyable_text.rs`
- `crates/agent_world_viewer/src/i18n.rs`

## 状态
- 当前阶段：已完成 VO1~VO9
- 下一阶段：按需继续优化信息密度、视觉分层与异常聚焦能力（重点是高风险事件前置与图文密度平衡）
- 最近更新：修复 3D 视口裁剪导致的半屏空白，并将右侧面板默认策略调优到观察优先（2026-02-09）
