# Viewer Web 闭环测试策略设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-web-closure-testing-policy.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-web-closure-testing-policy.project.md`

## 1. 设计定位
定义 `agent_world_viewer` 闭环测试默认切换到 Web 端的执行策略、产物要求与 fallback 边界，统一团队验证入口。

## 2. 设计结构
- 执行入口层：默认使用 `trunk serve + Playwright` 完成真实浏览器闭环。
- 证据产物层：要求保留 screenshot、console log 与最小可加载证据。
- 文档治理层：`AGENTS.md`、viewer manual、脚本说明统一回写为 Web 默认口径。
- 兼容 fallback 层：`capture-viewer-frame.sh` 保留，但仅在 native 图形链路或 Web 无法复现时使用。

## 3. 关键接口 / 入口
- `./scripts/run-viewer-web.sh --port 4173 --address 127.0.0.1`
- Playwright CLI 的 `open` / `snapshot` / `screenshot` / `console` / `eval`
- `output/playwright/*.png`
- `.playwright-cli/console-*.log`
- `scripts/capture-viewer-frame.sh`

## 4. 约束与边界
- Web 端是默认闭环路径，文档不得继续把 native 截图链路写成主流程。
- 最小通过标准固定为页面可加载、`console error = 0`、至少一张截图产物。
- fallback 保留但必须显式标注触发条件，避免团队双重口径。
- 本专题只调整流程规范，不扩展在线协议桥接或新增 viewer 功能。

## 5. 设计演进计划
- 先统一文档入口与执行口径。
- 再回写闭环命令、产物目录与最小验收标准。
- 最后以最小回归验证 Web 默认链路可执行，并保留 native fallback 应急方案。
