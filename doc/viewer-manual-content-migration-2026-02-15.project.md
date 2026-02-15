# Viewer 使用手册内容搬迁（2026-02-15）项目管理文档

## 任务拆解

### 0. 文档与基线
- [x] 新增设计文档（`doc/viewer-manual-content-migration-2026-02-15.md`）
- [x] 新增项目管理文档（本文件）
- [x] 明确搬迁来源清单（viewer-* + capture 脚本文档）

### 1. 中文基线手册搬迁
- [ ] 更新 `doc/viewer-manual.md`（新增操作章节）
- [ ] 清理冲突口径（保持 Web 默认 / native fallback）
- [ ] 自检章节结构与命令可复制性

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
- 当前阶段：进行中（M1 完成，任务 1 未开始）
- 最近更新：完成搬迁任务建模与文档拆解（2026-02-15）
- 下一步：执行任务 1，先更新中文基线手册。
