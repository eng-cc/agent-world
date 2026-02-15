# Site 使用手册静态化（CN/EN）项目管理文档

## 任务拆解

### 0. 文档与基线
- [x] 新增设计文档（`doc/site-manual-static-docs.md`）
- [x] 新增项目管理文档（本文件）
- [x] 明确内容基线（`doc/viewer-manual.md`）

### 1. 任务一：静态文档框架
- [x] 新增 `site/doc/cn/index.html` 与 `site/doc/en/index.html`
- [x] 新增文档页导航/语言切换基础布局
- [x] 在中英文首页接入“使用手册”入口
- [x] 补充文档框架样式与最小交互（含文档路径语言重定向保护）

### 2. 任务二：整理用户手册
- [x] 完善 `site/doc/cn/viewer-manual.html` 正文内容
- [x] 完善 `site/doc/en/viewer-manual.html` 正文内容
- [x] 目录页接入手册卡片与跳转
- [x] 校对中英文命令与链接一致性

### 3. 验证与收口
- [x] 执行 `env -u RUSTC_WRAPPER cargo check`（任务一）
- [x] 执行 `env -u RUSTC_WRAPPER cargo check`（任务二）
- [x] 更新项目管理文档状态
- [x] 写任务日志（`doc/devlog/2026-02-15-27.md`、`doc/devlog/2026-02-15-28.md`、`doc/devlog/2026-02-15-29.md`）

## 依赖
- 沿用现有 `site/` 静态部署与 `.github/workflows/pages.yml`。
- 以 `doc/viewer-manual.md` 作为用户手册内容基线。

## 状态
- 当前阶段：已完成
- 最近更新：完成 Viewer 手册中英文正文整理并发布到 `site/doc/cn|en`（2026-02-15）
- 下一步：后续按 `doc/viewer-manual.md` 变更滚动同步站内手册正文。
