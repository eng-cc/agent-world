# Viewer Texture Inspector 框架合理性优化（2026-03-01）（项目管理文档）

审计轮次: 3

## 任务拆解
- [x] T0：输出设计文档 `doc/scripts/viewer-tools/viewer-texture-inspector-framework-rationalization-2026-03-01.prd.md`
- [x] T0：输出项目管理文档（本文件）
- [x] T1：Rust 配置解析模块化（拆分 parsing 模块并控制 `viewer_3d_config.rs` 行数）
- [x] T2：Shell 构图策略结构化（power pose 统一解析入口）

## 依赖
- `crates/agent_world_viewer/src/viewer_3d_config.rs`
- `crates/agent_world_viewer/src/viewer_3d_config_profile_tests.rs`
- `scripts/viewer-texture-inspector.sh`
- `scripts/capture-viewer-frame.sh`

## 状态
- 当前阶段：已完成（T0 ~ T2 全部完成）
- 阻塞：无
- 下一步：无（后续新增实体时沿用统一 pose 解析入口扩展）

## 迁移记录（2026-03-03）
- 已按 `TASK-ENGINEERING-014-D1 (PRD-ENGINEERING-006)` 从 legacy 命名迁移为 `.prd.md/.prd.project.md`。
- 保留原任务拆解、依赖与状态语义，不改变既有结论。
