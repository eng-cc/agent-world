# Web UI Playwright 闭环测试手册（项目管理文档）

审计轮次: 3

## 任务拆解（含 PRD-ID 映射）
- [x] WPCM-1 (PRD-TESTING-WEB-001): 从主手册拆分 Web UI Playwright 闭环分册并建立唯一入口。
- [x] WPCM-2 (PRD-TESTING-WEB-001/003): 补齐启动前自检、会话防抖与 F1~F4 fail-fast 处置流程。
- [x] WPCM-3 (PRD-TESTING-WEB-002): 固化 GPU + headed 硬门禁与软件渲染阻断规则。
- [x] WPCM-4 (PRD-TESTING-WEB-002/003): 对齐 `viewer-release-qa-loop.sh` 与 `viewer-release-full-coverage.sh` 的门禁口径和产物要求。
- [x] WPCM-5 (PRD-TESTING-004): 专题文档人工迁移到 strict schema，并统一 `.prd.md/.prd.project.md` 命名。

## 依赖
- doc/testing/manual/web-ui-playwright-closure-manual.prd.md
- `testing-manual.md`
- `scripts/run-viewer-web.sh`
- `scripts/viewer-release-qa-loop.sh`
- `scripts/viewer-release-full-coverage.sh`
- `$CODEX_HOME/skills/playwright/scripts/playwright_cli.sh`（默认 `~/.codex/...`；仓库开发副本 `.agents/skills/playwright/scripts/playwright_cli.sh`）
- `doc/testing/manual/systematic-application-testing-manual.prd.md`
- `doc/testing/prd.md`
- `doc/testing/prd.project.md`

## 状态
- 更新日期：2026-03-03
- 当前阶段：已完成
- 阻塞项：无
- 下一步：无（manual 批次迁移收口）
