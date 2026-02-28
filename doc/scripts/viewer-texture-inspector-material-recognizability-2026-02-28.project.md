# Viewer Texture Inspector 材质可辨识增强（2026-02-28）（项目管理文档）

## 任务拆解
- [x] T0：输出设计文档 `doc/scripts/viewer-texture-inspector-material-recognizability-2026-02-28.md`
- [x] T0：输出项目管理文档（本文件）
- [ ] T1：为 `viewer-texture-inspector` 新增 `--preview-mode` 与 `lookdev` 模式（关闭 location shell/radiation/damage 干扰）
- [ ] T1：将 `preview_mode` 与 lookdev 开关写入 `meta.txt`
- [ ] T1：执行 test_tier_required（语法/help/smoke）并记录结论
- [ ] T2：实现 `direct_entity` 预览链路（直连被检实体槽位）
- [ ] T2：补充 `direct_entity` 失败回退与元数据字段
- [ ] T3：引入 per-entity/per-variant 参数层并接入脚本
- [ ] T4：引入变体级贴图槽位能力并完成小规模回归
- [ ] T5：完善评审矩阵与门禁收口，更新文档与日志结项

## 依赖
- `scripts/capture-viewer-frame.sh`（现有截图与自动化链路）
- `ffmpeg`（裁切与 SSIM）
- `env -u RUSTC_WRAPPER cargo run -p agent_world_viewer`（viewer 启动约束）

## 状态
- 当前阶段：进行中（T0 已完成，执行 T1）
- 阻塞：无
- 下一步：实现 `--preview-mode lookdev`，并完成语法/help/smoke 验证
