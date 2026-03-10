# Viewer Texture Inspector 框架合理性治理（2026-02-28）（项目管理文档）

- 对应设计文档: `doc/scripts/viewer-tools/viewer-texture-inspector-framework-rationalization-2026-02-28.design.md`
- 对应需求文档: `doc/scripts/viewer-tools/viewer-texture-inspector-framework-rationalization-2026-02-28.prd.md`

审计轮次: 4

## 任务拆解
- [x] T0：输出设计文档 `doc/scripts/viewer-tools/viewer-texture-inspector-framework-rationalization-2026-02-28.prd.md`
- [x] T0：输出项目管理文档（本文件）
- [x] T1：扩展 viewer capture status 语义可观测字段并补充测试
- [x] T2：落地 inspector 分层门禁（连接/语义/细节/差异）并写入元数据
- [x] T3：执行 power 场景回归矩阵并完成文档/日志结项

## 依赖
- `crates/agent_world_viewer/src/internal_capture.rs`
- `scripts/viewer-texture-inspector.sh`
- `scripts/capture-viewer-frame.sh`
- `ffmpeg`

## 状态
- 当前阶段：已完成（T0~T3）
- 阻塞：无
- 下一步：无

## 迁移记录（2026-03-03）
- 已按 `TASK-ENGINEERING-014-D1 (PRD-ENGINEERING-006)` 从 legacy 命名迁移为 `.prd.md/.project.md`。
- 保留原任务拆解、依赖与状态语义，不改变既有结论。
