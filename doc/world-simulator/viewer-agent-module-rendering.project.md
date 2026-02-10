# Viewer Agent 渲染改造（项目管理文档）

## 任务拆解
- [x] AMR-1：输出设计文档（`viewer-agent-module-rendering.md`）
- [x] AMR-2：输出项目管理文档（本文件）
- [ ] AMR-3：替换 Agent 主体渲染为胶囊体并对齐尺寸映射
- [ ] AMR-4：接入模块数量环带渲染（基于 installed slots）
- [ ] AMR-5：更新/补充单元测试（尺寸映射、模块数量映射）
- [ ] AMR-6：执行编译与测试回归（viewer 重点）
- [ ] AMR-7：执行截图闭环并确认效果
- [ ] AMR-8：更新开发日志与状态收口

## 依赖
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/scene_helpers.rs`
- `crates/agent_world_viewer/src/tests.rs`
- `scripts/capture-viewer-frame.sh`
- `doc/world-simulator/viewer-agent-module-rendering.md`

## 状态
- 当前阶段：文档阶段完成（AMR-1~AMR-2）。
- 下一阶段：实现渲染改造并完成闭环验证（AMR-3~AMR-8）。
- 最近更新：新增 Agent 渲染改造设计与项目文档（2026-02-10）。
