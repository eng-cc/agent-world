# Viewer 发行验收测试迭代闭环（完成度 + 视觉效果）

## 目标
- 基于现有 `testing-manual.md` 与 `window.__AW_TEST__`，建立一套可重复执行的发行验收闭环：自动验证功能完成度与视觉可用性。
- 将“发现套件问题 -> 修复套件 -> 重跑验证”流程产品化为脚本，减少手工命令拼接与漏测。
- 输出可归档的验收报告（通过项/失败项/证据路径），用于后续持续迭代。

## 范围

### In Scope
- 固化一条可执行链路：`viewer-visual-baseline` + Web 语义化闭环（Playwright + `__AW_TEST__`）。
- 对 Web 闭环增加语义断言：
  - `__AW_TEST__` 可用性
  - 连接状态/tick 进展
  - 基本操控动作（`runSteps`/`sendControl`/`getState`）可达
  - 截图与控制台证据落盘
- 生成 QA 汇总报告（Markdown），记录完成度与视觉门禁结果。

### Out of Scope
- 不在本轮重写 `agent_world_viewer` 渲染表现或玩法逻辑。
- 不引入完整 CI 常驻 E2E（本轮先以本地/agent 闭环脚本为主）。
- 不替代已有 `test_tier_required/full`，仅作为发行验收补充层。

## 接口 / 数据

### 验收脚本入口
- 计划新增脚本：`scripts/viewer-release-qa-loop.sh`
- 关键参数（计划）：
  - `--scenario <name>`：默认 `llm_bootstrap`
  - `--viewer-port <port>`：默认 `4173`
  - `--web-bridge-port <port>`：默认 `5011`
  - `--tick-ms <ms>`：默认 `300`
  - `--out-dir <dir>`：默认 `output/playwright/viewer`

### 语义化断言（Web）
- `window.__AW_TEST__` 存在。
- `getState()` 返回关键字段：`connectionStatus`、`tick`。
- 执行动作序列后状态保持可读且无明显错误放大：
  - `runSteps("mode=3d;focus=first_location;zoom=0.85;select=first_agent;wait=0.3")`
  - `sendControl("pause")`
  - `sendControl("play")`

### 产物
- 截图：`output/playwright/viewer/*.png`
- 控制台：`.playwright-cli/console-*.log`
- 汇总报告：`output/playwright/viewer/release-qa-summary-*.md`

## 里程碑
- VRQ-0：文档建档（设计 + 项目管理）
- VRQ-1：执行现有套件基线，确认完成度与视觉门禁现状
- VRQ-2：实现一键化 QA 脚本并补齐语义断言
- VRQ-3：重跑验证，输出结论并更新手册/项目状态

## 风险
- 浏览器启动与端口占用风险：
  - 缓解：脚本内统一拉起/清理进程，显式端口参数化。
- Web 状态异步波动导致误判：
  - 缓解：加入轮询等待与超时机制，不用瞬时状态判定通过。
- Playwright/Node 环境差异风险：
  - 缓解：复用仓库既有 `playwright_cli.sh` 包装层，保持 Node 版本前置检查。
