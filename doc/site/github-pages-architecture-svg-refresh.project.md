# GitHub Pages 架构图 SVG 精修（四期增量）项目管理文档

## 任务拆解

### 0. 文档与基线
- [x] 新增增量设计文档（`doc/site/github-pages-architecture-svg-refresh.md`）
- [x] 新增增量项目管理文档（本文件）
- [x] 明确验收标准（层次清晰/科技感提升/引用不变）

### 1. SVG 重绘
- [x] 重构主流程节点布局（Runtime -> Simulator -> Viewer -> LLM/Modules）
- [x] 重绘底层能力区（Persistence/Evidence 与 Control/Feedback）
- [x] 优化背景与连接线风格（网格、光带、箭头）

### 2. 验证与收口
- [x] 校验 SVG 语义标签与结构完整
- [x] 执行 `env -u RUSTC_WRAPPER cargo check`
- [x] 更新项目管理文档状态
- [x] 写入当日开发日志（`doc/devlog/2026-02-12.md`）

## 依赖
- 沿用现有页面引用路径，不改 HTML 结构。
- 不新增外部字体或图片依赖。

## 状态
- 当前阶段：已完成
- 最近更新：完成架构图 SVG 重绘、结构校验与 `cargo check`（2026-02-12）
- 下一步：可按演示需求追加“简化版小尺寸架构图”（社媒卡片专用）。
