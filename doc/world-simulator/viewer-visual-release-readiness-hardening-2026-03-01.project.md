# Viewer 视觉外发就绪硬化（项目管理）

## 任务拆解
- [x] VVRH-0 文档建档：设计文档 + 项目管理文档
- [x] VVRH-1 视觉门禁硬化：texture inspector strict gate + 选中/构图修复
- [x] VVRH-2 发布向 UI profile：隐藏调试干扰并固化脚本接入
- [x] VVRH-3 工业主题资产升级：industrial_v3 + 校验阈值更新
- [ ] VVRH-4 外发样张基线：样张脚本、产物留痕、手册与状态收口

## 依赖
- `scripts/viewer-texture-inspector.sh`
- `scripts/viewer-theme-pack-preview.sh`
- `scripts/validate-viewer-theme-pack.py`
- `scripts/generate-viewer-industrial-theme-assets.py`
- `crates/agent_world_viewer/assets/themes/`
- `testing-manual.md`

## 状态
- 当前阶段：进行中（VVRH-0 ~ VVRH-3 已完成，执行 VVRH-4）。
- 发布判定目标：全部任务完成后再重新评估“可对外发布”结论。
- 最近更新：2026-03-01。
