# Viewer 贴图查看器（项目管理文档）

审计轮次: 3

## 任务拆解（含 PRD-ID 映射）
- [x] VTI-0 文档建档：设计文档 + 项目管理文档
- [x] VTI-1 脚本实现：贴图查看器 + 截图导出
- [x] VTI-2 测试与收口：手册、devlog、状态回写

## 依赖
- doc/world-simulator/viewer/viewer-texture-inspector.prd.md
- `scripts/capture-viewer-frame.sh`
- `scripts/viewer-theme-pack-preview.sh`
- `crates/agent_world_viewer/assets/themes/industrial_v1/presets/`
- `doc/world-simulator/viewer/viewer-manual.md`
- `testing-manual.md`

## 状态
- 当前阶段：VTI 全部任务已完成（VTI-0 ~ VTI-2）。
- 下一步：若需要更高信息密度，可补“多机位拼图导出（wide + close-up）”。 
- 最近更新：2026-02-21（完成手册接入与收口测试）。
