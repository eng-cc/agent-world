# GitHub Pages 内容状态同步（2026-02-25）设计文档

## 目标
- 基于当前仓库文档与代码现状，修正 GitHub Pages 中已过时的信息口径。
- 保持中英文页面同构，确保首页与手册页对同一能力结论一致。
- 在纯静态站（HTML/CSS/JS）前提下完成更新，不引入构建链路。

## 范围
- 范围内
  - 更新首页（`site/index.html`、`site/en/index.html`）中的近期更新与运行说明文案。
  - 更新手册静态页（`site/doc/cn|en/viewer-manual.html`）以匹配当前实现能力：
    - `world_viewer_live` 默认 LLM、`--no-llm` 回退。
    - `--release-config` 锁定参数模式。
    - Web Test API `sendControl("step")` 语义。
    - 自动化 target 通用语法（`first:<kind>` / `<kind>:<id>`）。
  - 更新文档目录页（`site/doc/cn|en/index.html`）中的手册状态摘要。
- 范围外
  - 改动页面视觉体系、导航结构或交互框架。
  - 引入新的静态站生成器或前端依赖。
  - 修改后端运行时协议。

## 接口/数据
- 输入基线
  - `doc/viewer-manual.md`
  - `doc/world-simulator.project.md`
  - `doc/world-simulator/viewer-generic-focus-targets.md`
  - `doc/world-simulator/viewer-web-test-api-step-control-2026-02-24.md`
  - `crates/agent_world/src/bin/world_viewer_live/cli.rs`
  - `crates/agent_world_viewer/src/web_test_api.rs`
  - `crates/agent_world_viewer/src/viewer_automation.rs`
- 输出文件
  - `site/index.html`
  - `site/en/index.html`
  - `site/doc/cn/index.html`
  - `site/doc/en/index.html`
  - `site/doc/cn/viewer-manual.html`
  - `site/doc/en/viewer-manual.html`

## 里程碑
- M1：文档与任务拆解
  - 新增本设计文档与项目管理文档。
- M2：Pages 内容同步
  - 完成首页与手册页中英文口径更新。
- M3：验证与收口
  - 完成 `cargo check`、项目文档状态回写、当日 devlog 记录。

## 风险
- 风险：中英文文案更新不一致。
  - 缓解：同一任务内成对维护 `cn/en` 页面并对齐章节。
- 风险：手册页超出当前可维护密度。
  - 缓解：仅同步能力口径，不扩展新章节结构。
- 风险：代码能力继续演进导致页面再次过时。
  - 缓解：以 `doc/viewer-manual.md` 作为后续同步基线，滚动更新。
