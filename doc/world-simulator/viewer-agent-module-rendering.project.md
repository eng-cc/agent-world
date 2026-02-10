# Viewer Agent 渲染改造（项目管理文档）

## 任务拆解
- [x] AMR-1：输出设计文档（`viewer-agent-module-rendering.md`）
- [x] AMR-2：输出项目管理文档（本文件）
- [x] AMR-3：替换 Agent 主体渲染为胶囊体并对齐尺寸映射
- [x] AMR-4：接入模块数量渲染（模块节点环 + 数量立柱）
- [x] AMR-5：更新/补充单元测试（尺寸映射、模块数量映射、模块上限渲染）
- [x] AMR-6：执行编译与测试回归（viewer 重点）
- [x] AMR-7：执行截图闭环并确认效果
- [x] AMR-8：更新开发日志与状态收口
- [x] AMR-9：模块渲染升级为立方体拼接机器人布局
- [x] AMR-10：补充/调整测试并完成闭环截图复核
- [x] AMR-11：更新日志并收口本轮任务

## 依赖
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/scene_helpers.rs`
- `crates/agent_world_viewer/src/tests.rs`
- `scripts/capture-viewer-frame.sh`
- `doc/world-simulator/viewer-agent-module-rendering.md`

## 状态
- 当前阶段：增强阶段已完成（AMR-9~AMR-11 全部完成）。
- 下一阶段：可选继续优化（2D/3D 视角差异化展示策略、模块类型分色图例、可配置夸张系数开关）。
- 最近更新：完成“模块立方体拼接机器人”闭环验证并收口（2026-02-10）；并将 Agent 贴附策略修正为严格位于 Location 表面。
