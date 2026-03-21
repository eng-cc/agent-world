# Viewer Texture Inspector 材质可辨识增强（2026-02-28）（项目管理文档）

- 对应设计文档: `doc/scripts/viewer-tools/viewer-texture-inspector-material-recognizability-2026-02-28.design.md`
- 对应需求文档: `doc/scripts/viewer-tools/viewer-texture-inspector-material-recognizability-2026-02-28.prd.md`

审计轮次: 4

## 任务拆解
- [x] T0：输出设计文档 `doc/scripts/viewer-tools/viewer-texture-inspector-material-recognizability-2026-02-28.prd.md`
- [x] T0：输出项目管理文档（本文件）
- [x] T1：为 `viewer-texture-inspector` 新增 `--preview-mode` 与 `lookdev` 模式（关闭 location shell/radiation/damage 干扰）
- [x] T1：将 `preview_mode` 与 lookdev 开关写入 `meta.txt`
- [x] T1：执行 test_tier_required（语法/help/smoke）并记录结论
- [x] T2：实现 `direct_entity` 预览链路（直连被检实体槽位）
- [x] T2：补充 `direct_entity` 失败回退与元数据字段
- [x] T3：引入 per-entity/per-variant 参数层并接入脚本
- [x] T4：引入变体级贴图槽位能力并完成小规模回归
- [x] T5：完善评审矩阵与门禁收口，更新文档与日志结项

## 依赖
- `scripts/capture-viewer-frame.sh`（现有截图与自动化链路）
- `ffmpeg`（裁切与 SSIM）
- `env -u RUSTC_WRAPPER cargo run -p oasis7_viewer`（viewer 启动约束）

## 状态
- 当前阶段：已完成（T0~T5）
- 阻塞：无
- 下一步：无（本项目任务已结项）

## 迁移记录（2026-03-03）
- 已按 `TASK-ENGINEERING-014-D1 (PRD-ENGINEERING-006)` 从 legacy 命名迁移为 `.prd.md/.project.md`。
- 保留原任务拆解、依赖与状态语义，不改变既有结论。
