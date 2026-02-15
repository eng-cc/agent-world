# Viewer Web 端闭环测试策略

## 目标
- 将 `agent_world_viewer` 的闭环测试默认路径统一到 Web 端：`trunk serve + Playwright`。
- 把“可复现、可自动化、可留证据”作为闭环基线，减少桌面端窗口捕获差异导致的不稳定性。
- 保留原生截图链路作为历史兼容/应急方案，但不再作为默认流程。

## 范围
- 范围内：
  - 更新团队文档中的闭环测试标准流程，统一改为 Web 端。
  - 在手册中明确 Web 闭环命令、产物目录与最小验收口径（console error=0 + screenshot）。
  - 将 `capture-viewer-frame` 流程标记为 fallback（legacy）。
- 范围外：
  - 不移除现有 `scripts/capture-viewer-frame.sh` 实现。
  - 不在本任务实现 Web 在线协议桥接（浏览器连 `world_viewer_live`）。
  - 不新增前端功能，仅调整流程规范与文档口径。

## 接口 / 数据

### 1) 默认闭环入口
- 启动 Web viewer：`./scripts/run-viewer-web.sh --port 4173 --address 127.0.0.1`
- 闭环自动化：Playwright CLI（建议 Node 20+）
  - `open/snapshot/screenshot/console/eval`

### 2) 闭环产物标准
- 截图：`output/playwright/*.png`
- 控制台日志：`.playwright-cli/console-*.log`
- 通过基线：
  - 页面可加载（title/canvas 存在）
  - `console error = 0`
  - 至少 1 张截图产物

### 3) 兼容链路
- `scripts/capture-viewer-frame.sh` 继续保留，定位为：
  - native 图形链路故障排查
  - Web 端无法复现的特定问题

## 里程碑
- WCT1：输出策略设计文档与项目管理文档。
- WCT2：完成相关文档批量迁移（AGENTS/手册/脚本文档/Web 文档）。
- WCT3：完成最小回归验证并更新状态、日志、提交收口。

## 风险
- Node/Playwright 版本差异风险：旧 Node 版本可能导致 Playwright CLI 不稳定。
  - 缓解：文档显式要求 Node 20+，并提供版本检查命令。
- Web 离线模式边界风险：当前浏览器端默认离线。
  - 缓解：在文档中明确“闭环主要验证渲染/UI/交互稳定性”，在线链路单独规划。
- 历史习惯迁移风险：团队仍可能使用 native 截图脚本。
  - 缓解：在关键入口文档中统一声明“Web 为默认，native 为 fallback”。
