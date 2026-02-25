# GitHub Pages 首页“游戏优先”分层重构（2026-02-25）项目管理文档

## 任务拆解

### 0. 文档与基线
- [x] 新增设计文档（`doc/site/github-pages-game-first-home-2026-02-25.md`）
- [x] 新增项目管理文档（本文件）
- [x] 明确改动范围仅限首页 CN/EN

### 1. 首页重排（CN/EN）
- [x] 重排 `site/index.html`，将游戏叙事置于前半屏
- [x] 重排 `site/en/index.html`，保持与中文同构
- [x] 将引擎能力收拢到底部几屏（架构/引擎扩展区）

### 2. 验证与收口
- [x] 执行 `env -u RUSTC_WRAPPER cargo check -p agent_world_viewer`
- [x] 回写本项目管理文档状态
- [x] 写任务日志（`doc/devlog/2026-02-25.md`）

## 依赖
- 沿用 `site/` 静态页面与现有 CSS/JS 交互。
- 交互标记保持 `site/assets/app.js` 兼容。

## 状态
- 当前阶段：已完成（任务 0/1/2 全部完成）
- 最近更新：完成验证收口并结项（2026-02-25）
- 下一步：无。
