# Viewer Texture Inspector 视觉细节系统优化（2026-02-28）（项目管理文档）

## 任务拆解
- [x] T0：输出设计文档 `doc/scripts/viewer-texture-inspector-visual-detail-system-optimization-2026-02-28.md`
- [x] T0：输出项目管理文档（本文件）
- [x] T1：viewer 启动层支持 `AGENT_WORLD_VIEWER_PANEL_HIDDEN` 并补测试
- [x] T2：inspector 落地构图候选策略 + 资源包 + 灯光预设 + 元数据
- [x] T3：执行 power 场景回归并完成结项文档/日志
- [x] T4：设施实体尺度归一化 + direct_entity 隔离/构图修复并复跑视觉回归
- [ ] T5：材质差异增强（power_plant/power_storage）并完成阈值回归

## 依赖
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `scripts/viewer-texture-inspector.sh`
- `scripts/capture-viewer-frame.sh`
- `ffmpeg`

## 状态
- 当前阶段：进行中（T4 已完成，执行 T5）
- 阻塞：无
- 下一步：在保持 direct_entity 可读性的前提下，系统化拉开 matte/glossy/default 的画面差异，压低跨变体 SSIM
