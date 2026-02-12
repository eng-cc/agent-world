# GitHub Pages 架构图 SVG 精修（四期增量）项目管理文档

## 任务拆解

### 0. 文档与基线
- [x] 新增增量设计文档（`doc/github-pages-architecture-svg-refresh.md`）
- [x] 新增增量项目管理文档（本文件）
- [x] 明确验收标准（层次清晰/科技感提升/引用不变）

### 1. SVG 重绘
- [ ] 重构主流程节点布局（Runtime -> Simulator -> Viewer -> LLM/Modules）
- [ ] 重绘底层能力区（Persistence/Evidence 与 Control/Feedback）
- [ ] 优化背景与连接线风格（网格、光带、箭头）

### 2. 验证与收口
- [ ] 校验 SVG 语义标签与结构完整
- [ ] 执行 `env -u RUSTC_WRAPPER cargo check`
- [ ] 更新项目管理文档状态
- [ ] 写入当日开发日志（`doc/devlog/2026-02-12.md`）

## 依赖
- 沿用现有页面引用路径，不改 HTML 结构。
- 不新增外部字体或图片依赖。

## 状态
- 当前阶段：执行中
- 最近更新：完成架构图精修文档初始化（2026-02-12）
- 下一步：完成 SVG 图稿重绘与验证收口。
