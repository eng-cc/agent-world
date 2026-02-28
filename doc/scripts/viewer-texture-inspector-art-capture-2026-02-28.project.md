# Viewer Texture Inspector Art Capture 优化（2026-02-28）（项目管理文档）

## 任务拆解
- [x] T0：输出设计文档 `doc/scripts/viewer-texture-inspector-art-capture-2026-02-28.md`
- [x] T0：输出项目管理文档（本文件）
- [x] T1：实现 `--art-capture` 模式与分实体镜头策略
- [x] T1：输出 `viewer_art.png` 与元数据（镜头/裁切参数）
- [x] T1：补充/更新脚本帮助信息
- [x] T2：执行 test_tier_required（语法/help/最小实跑）
- [x] T2：更新项目管理文档状态与 `doc/devlog/2026-02-28.md`
- [x] T3：实现 closeup 双图产物（`viewer_closeup.png`/`viewer_art_closeup.png`）与 closeup 元数据
- [x] T3：实现 `power_plant/power_storage` 三变体一致性校验、重试与 `variant_validation.txt`
- [x] T4：修复镜头贴脸问题（重标定各实体 `hero/closeup` 默认镜头）
- [x] T4：执行全量复跑（15 组合）并完成视觉检查结论
- [x] T5：为 power 实体改造焦点目标（`first_power_plant` / `first_power_storage`）
- [x] T5：新增/接线 art-lighting 评审灯光口径（可开关）
- [x] T5：新增 SSIM 阈值门禁（含重试、元数据、`variant_validation.txt` 字段）
- [x] T5：执行 smoke + 全量复跑并记录视觉检查结论

## 依赖
- `scripts/capture-viewer-frame.sh`（现有截图链路）
- `ffmpeg`（截图与裁切）
- `env -u RUSTC_WRAPPER cargo run`（viewer/live 运行约束）

## 状态
- 当前阶段：已完成（T0~T5）
- 阻塞：无
- 下一步：无（本轮任务已结项）
