# Web UI agent-browser 闭环测试手册（项目管理文档）

- 对应设计文档: `doc/testing/manual/web-ui-playwright-closure-manual.design.md`
- 对应需求文档: `doc/testing/manual/web-ui-agent-browser-closure-manual.prd.md`

审计轮次: 4

## 任务拆解（含 PRD-ID 映射）
- [x] WPCM-1 (PRD-TESTING-WEB-001): 从主手册拆分 Web UI agent-browser 闭环分册并建立唯一入口。
- [x] WPCM-2 (PRD-TESTING-WEB-001/003): 补齐启动前自检、会话防抖与 F1~F4 fail-fast 处置流程。
- [x] WPCM-3 (PRD-TESTING-WEB-002): 固化 GPU + headed 硬门禁与软件渲染阻断规则。
- [x] WPCM-4 (PRD-TESTING-WEB-002/003): 对齐 `viewer-release-qa-loop.sh` 与 `viewer-release-full-coverage.sh` 的门禁口径和产物要求。
- [x] WPCM-5 (PRD-TESTING-004): 专题文档人工迁移到 strict schema，并统一 `.prd.md/.project.md` 命名。
- [x] WPCM-6 (PRD-TESTING-WEB-001/002): 明确 Viewer 页面与 `world_web_launcher` 控制面的执行边界：前者 `agent-browser` 优先，后者 GUI Agent 优先。
- [x] WPCM-7 (PRD-TESTING-WEB-002): 固化 Viewer Web 的默认硬件 WebGL 启动参数，并明确 headed 若仍落到 software renderer 继续按环境阻断。

## 依赖
- doc/testing/manual/web-ui-agent-browser-closure-manual.prd.md
- `testing-manual.md`
- `scripts/run-viewer-web.sh`
- `scripts/viewer-release-qa-loop.sh`
- `scripts/viewer-release-full-coverage.sh`
- `agent-browser` CLI（二进制命令；默认直接通过 `PATH` 调用）
- `scripts/agent-browser-lib.sh`
- `doc/testing/manual/systematic-application-testing-manual.prd.md`
- `doc/testing/prd.md`
- `doc/testing/project.md`

## 状态
- 更新日期：2026-03-12
- 当前阶段：已完成（已补充 headed + hardware WebGL 固定策略）
- 阻塞项：无
- 下一步：继续观察不同图形环境下是否仍有默认 ANGLE 回退到 SwiftShader 的特例。
