# Viewer Texture Inspector 框架合理性优化（2026-03-01）（项目管理文档）

## 任务拆解
- [x] T0：输出设计文档 `doc/scripts/viewer-texture-inspector-framework-rationalization-2026-03-01.md`
- [x] T0：输出项目管理文档（本文件）
- [ ] T1：Rust 配置解析模块化（拆分 parsing 模块并控制 `viewer_3d_config.rs` 行数）
- [ ] T2：Shell 构图策略结构化（power pose 统一解析入口）

## 依赖
- `crates/agent_world_viewer/src/viewer_3d_config.rs`
- `crates/agent_world_viewer/src/viewer_3d_config_profile_tests.rs`
- `scripts/viewer-texture-inspector.sh`
- `scripts/capture-viewer-frame.sh`

## 状态
- 当前阶段：T0 已完成，T1 进行中
- 阻塞：无
- 下一步：完成 T1 代码与验证并提交
