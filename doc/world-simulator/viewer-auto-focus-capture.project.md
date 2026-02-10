# Agent World Simulator：Viewer 自动聚焦（人机共用 + 截图闭环）（项目管理文档）

## 任务拆解
- [x] AFC1：输出设计文档与项目管理文档
- [x] AFC2：viewer 自动聚焦（启动配置 + `F` 快捷键）
- [x] AFC3：截图脚本参数接入自动聚焦
- [x] AFC4：测试回归与截图闭环验证
- [x] AFC5：文档收口与 devlog 更新

## 依赖
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/camera_controls.rs`
- `scripts/capture-viewer-frame.sh`

## 状态
- 当前阶段：已完成
- 最近更新：完成 AFC5（闭环截图验证通过 + 文档收口，2026-02-10）
