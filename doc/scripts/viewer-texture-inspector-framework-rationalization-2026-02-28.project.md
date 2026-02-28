# Viewer Texture Inspector 框架合理性治理（2026-02-28）（项目管理文档）

## 任务拆解
- [x] T0：输出设计文档 `doc/scripts/viewer-texture-inspector-framework-rationalization-2026-02-28.md`
- [x] T0：输出项目管理文档（本文件）
- [x] T1：扩展 viewer capture status 语义可观测字段并补充测试
- [ ] T2：落地 inspector 分层门禁（连接/语义/细节/差异）并写入元数据
- [ ] T3：执行 power 场景回归矩阵并完成文档/日志结项

## 依赖
- `crates/agent_world_viewer/src/internal_capture.rs`
- `scripts/viewer-texture-inspector.sh`
- `scripts/capture-viewer-frame.sh`
- `ffmpeg`

## 状态
- 当前阶段：进行中（T1 已完成，执行 T2）
- 阻塞：无
- 下一步：落地 inspector 分层门禁（连接/语义/细节/差异）并写入元数据
