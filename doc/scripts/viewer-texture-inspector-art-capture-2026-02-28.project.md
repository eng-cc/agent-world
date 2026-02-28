# Viewer Texture Inspector Art Capture 优化（2026-02-28）（项目管理文档）

## 任务拆解
- [x] T0：输出设计文档 `doc/scripts/viewer-texture-inspector-art-capture-2026-02-28.md`
- [x] T0：输出项目管理文档（本文件）
- [x] T1：实现 `--art-capture` 模式与分实体镜头策略
- [x] T1：输出 `viewer_art.png` 与元数据（镜头/裁切参数）
- [x] T1：补充/更新脚本帮助信息
- [x] T2：执行 test_tier_required（语法/help/最小实跑）
- [x] T2：更新项目管理文档状态与 `doc/devlog/2026-02-28.md`

## 依赖
- `scripts/capture-viewer-frame.sh`（现有截图链路）
- `ffmpeg`（截图与裁切）
- `env -u RUSTC_WRAPPER cargo run`（viewer/live 运行约束）

## 状态
- 当前阶段：已完成（T0~T2）
- 阻塞：无
- 下一步：无（本轮任务已结项）
