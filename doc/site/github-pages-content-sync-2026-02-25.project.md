# GitHub Pages 内容状态同步（2026-02-25）项目管理文档

## 任务拆解

### 0. 文档与基线
- [x] 新增设计文档（`doc/site/github-pages-content-sync-2026-02-25.md`）
- [x] 新增项目管理文档（本文件）
- [x] 明确输入基线（viewer 手册、world 项目状态、CLI 与 Web API 实现）

### 1. 首页与文档目录同步
- [ ] 更新 `site/index.html` 与 `site/en/index.html` 的近期更新与运行口径
- [ ] 更新 `site/doc/cn/index.html` 与 `site/doc/en/index.html` 的手册状态摘要
- [ ] 校对中英文锚点与入口链接一致性

### 2. 手册正文同步
- [ ] 更新 `site/doc/cn/viewer-manual.html`
- [ ] 更新 `site/doc/en/viewer-manual.html`
- [ ] 补齐默认 LLM/`--no-llm`、`--release-config`、Web step 控制、通用 target 语法

### 3. 验证与收口
- [ ] 执行 `env -u RUSTC_WRAPPER cargo check`
- [ ] 更新本项目管理文档状态
- [ ] 写任务日志（`doc/devlog/2026-02-25.md`）

## 依赖
- 继续沿用 `site/` 静态目录与 GitHub Pages 工作流。
- 内容基线以 `doc/viewer-manual.md` 和已合入代码行为为准。

## 状态
- 当前阶段：进行中（已完成任务 0）
- 最近更新：完成设计/项目文档建档并冻结本轮同步范围（2026-02-25）
- 下一步：执行任务 1 与任务 2 的中英文页面内容同步。
