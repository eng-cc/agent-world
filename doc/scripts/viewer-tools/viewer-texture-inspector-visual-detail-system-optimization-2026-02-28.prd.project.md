# Viewer Texture Inspector 视觉细节系统优化（2026-02-28）（项目管理文档）

审计轮次: 4

## 任务拆解
- [x] T0：输出设计文档 `doc/scripts/viewer-tools/viewer-texture-inspector-visual-detail-system-optimization-2026-02-28.prd.md`
- [x] T0：输出项目管理文档（本文件）
- [x] T1：viewer 启动层支持 `AGENT_WORLD_VIEWER_PANEL_HIDDEN` 并补测试
- [x] T2：inspector 落地构图候选策略 + 资源包 + 灯光预设 + 元数据
- [x] T3：执行 power 场景回归并完成结项文档/日志
- [x] T4：设施实体尺度归一化 + direct_entity 隔离/构图修复并复跑视觉回归
- [x] T5：材质差异增强（power_plant/power_storage）并完成阈值回归

## 依赖
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `scripts/viewer-texture-inspector.sh`
- `scripts/capture-viewer-frame.sh`
- `ffmpeg`

## 状态
- 当前阶段：已完成（T0 ~ T5 全部完成）
- 阻塞：无
- 下一步：无（如新增实体，沿用“实体级材质通道 + safe-radius 构图 profile”扩展）

## 迁移记录（2026-03-03）
- 已按 `TASK-ENGINEERING-014-D1 (PRD-ENGINEERING-006)` 从 legacy 命名迁移为 `.prd.md/.prd.project.md`。
- 保留原任务拆解、依赖与状态语义，不改变既有结论。
