# Viewer 使用手册内容搬迁（2026-02-15）项目管理文档

## 任务拆解

### 0. 文档与基线
- [x] 新增设计文档（`doc/viewer-manual-content-migration-2026-02-15.md`）
- [x] 新增项目管理文档（本文件）
- [x] 明确搬迁来源清单（viewer-* + capture 脚本文档）

### 1. 中文基线手册搬迁
- [x] 更新 `doc/viewer-manual.md`（新增操作章节）
- [x] 清理冲突口径（保持 Web 默认 / native fallback）
- [x] 自检章节结构与命令可复制性

### 2. 站点手册同步
- [ ] 更新 `site/doc/cn/viewer-manual.html`
- [ ] 更新 `site/doc/en/viewer-manual.html`
- [ ] 确认 CN/EN 章节对齐与链接可达

### 3. 验证与收口
- [ ] 执行 `env -u RUSTC_WRAPPER cargo check`
- [ ] 更新项目管理文档状态
- [ ] 写任务日志（`doc/devlog/2026-02-15-30.md`、`doc/devlog/2026-02-15-31.md`、`doc/devlog/2026-02-15-32.md`）

## 依赖
- 以 `doc/viewer-manual.md` 为中文基线。
- 站点发布页面位于 `site/doc/cn|en/viewer-manual.html`。

## 状态
- 当前阶段：进行中（任务 1 已完成，任务 2 未开始）
- 最近更新：完成 `doc/viewer-manual.md` 搬迁增强（自动步骤/详情面板/模块显隐/全览图缩放等）（2026-02-15）
- 下一步：执行任务 2，同步 `site/doc/cn|en/viewer-manual.html`。
