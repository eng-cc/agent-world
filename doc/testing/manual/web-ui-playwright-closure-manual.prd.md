# Agent World：Web UI Playwright 闭环测试手册（2026-02-28）

## 1. Executive Summary
- Problem Statement: Web UI 验收若缺少统一启动、采样、门禁与故障分级，容易出现“看起来可用但证据不可复现”的假通过。
- Proposed Solution: 建立 Playwright Web 闭环唯一手册，强制 GPU + headed 口径、语义化 `__AW_TEST__` 采样和 fail-fast 分级处置，并接入发布脚本。
- Success Criteria:
  - SC-1: S6 Web 闭环流程可由手册命令一键复现。
  - SC-2: 验收口径强制 `open ... --headed`，并阻断 `SwiftShader/software rendering`。
  - SC-3: 至少输出 `snapshot + console + screenshot + state` 证据。
  - SC-4: 发布验收脚本（`viewer-release-qa-loop/full-coverage`）可直接复用手册约束。
  - SC-5: 文档迁移后统一 `.prd.md/.prd.project.md` 命名并通过治理检查。

## 2. User Experience & Functionality
- User Personas:
  - Web 闭环执行者：按手册运行 Playwright 并归档证据。
  - 发布负责人：用一键脚本快速判断是否可放行。
  - 故障值守人员：根据 fail-fast 等级快速定位问题归属。
- User Scenarios & Frequency:
  - 每次 Viewer Web 相关改动后执行 S6 smoke。
  - 每次候选发布前执行 `viewer-release-qa-loop.sh`。
  - 关键版本执行 `viewer-release-full-coverage.sh`（含玩法与视觉）。
  - 连接失败或渲染崩溃时按 F1~F4 处置并归档证据。
- User Stories:
  - PRD-TESTING-WEB-001: As a Web 闭环执行者, I want deterministic startup and sampling commands, so that I can reproduce browser behavior reliably.
  - PRD-TESTING-WEB-002: As a 发布负责人, I want hard GPU/headed gate and clear pass criteria, so that release decisions are defensible.
  - PRD-TESTING-WEB-003: As a 故障值守人员, I want fail-fast taxonomy with actions, so that incident triage is fast and consistent.
- Critical User Flows:
  1. Flow-WEB-001: `启动 world_game_launcher -> 端口/主页自检 -> 打开 Web 页`
  2. Flow-WEB-002: `执行 Playwright 语义步骤 -> 采集状态/日志/截图 -> 关闭会话`
  3. Flow-WEB-003: `执行 GPU/headed 硬门禁 -> 检测软件渲染关键字 -> pass/fail`
  4. Flow-WEB-004: `触发 F1~F4 -> 输出分级结论 -> 归档证据并阻断放行`
  5. Flow-WEB-005: `运行 qa-loop/full-coverage -> 汇总产物 -> 发布评审`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 启动与自检 | `--live-bind`、`--web-bind`、viewer URL、端口监听 | 启动 launcher 并检查 4173/5011 与主页可达 | `booting -> ready` | 先端口后 URL，再进入采样 | 执行者可操作 |
| GPU 硬门禁 | `--headed`、console 关键字 (`SwiftShader` 等) | 采样前执行硬门禁检查 | `gating -> pass/fail` | 发现软件渲染即 fail | 发布/测试共同遵循 |
| Playwright 采样 | `snapshot`、`eval`、`console`、`screenshot`、`getState` | 基于 `__AW_TEST__` 执行语义步骤 | `sampling -> evidence` | 至少 1 张截图 + state 字段完整 | 执行者产出，发布者审阅 |
| 会话防抖 | `close-all`、fail-fast 预检查 | 每轮清理残留会话并快速失败 | `cleanup -> opened -> stable` | 先清会话后 open，减少残留干扰 | 执行者维护 |
| 发行验收脚本 | `viewer-release-qa-loop.sh`、`viewer-release-full-coverage.sh`、`--quick` | 一键执行发布门禁并输出总结 | `running -> summarized` | 先 QA loop，再 full coverage | 发布负责人触发 |
| 故障分级 | F1~F4 签名、处置动作、证据清单 | 识别错误并匹配处置流程 | `detected -> triaged -> archived` | 连接问题优先于可玩性判定 | 值守与维护者执行 |
- Acceptance Criteria:
  - AC-1: 手册提供可直接复制的启动/采样/门禁命令。
  - AC-2: 明确禁止 headless 验收与软件渲染口径。
  - AC-3: 定义最小通过标准（canvas、`__AW_TEST__`、`console error=0`、截图）。
  - AC-4: 提供 F1~F4 分级与对应处置动作。
  - AC-5: 发布脚本产物路径与门禁规则可追溯到本手册。
  - AC-6: 本专题迁移后引用更新到新命名并通过治理检查。
- Non-Goals:
  - 不在本专题替代 native 抓图应急链路。
  - 不在本专题重构 Viewer 业务逻辑或渲染实现。
  - 不在本专题扩展非 Web 场景测试规范。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: Playwright CLI 包装脚本（`$REPO_ROOT/.codex/skills/playwright/scripts/playwright_cli.sh`）用于浏览器自动化；其余为测试脚本与终端命令。
- Evaluation Strategy: 通过语义动作成功率（`__AW_TEST__` 可用性）、门禁通过率和故障分级命中率评估闭环质量。

## 4. Technical Specifications
- Architecture Overview: Web 闭环由 `world_game_launcher` 提供 live/runtime + web 静态服务，Playwright CLI 负责页面驱动与证据采样，发布脚本承载标准化验收与总结。
- Integration Points:
  - `testing-manual.md`
  - `scripts/run-viewer-web.sh`
  - `scripts/viewer-release-qa-loop.sh`
  - `scripts/viewer-release-full-coverage.sh`
  - `.codex/skills/playwright/scripts/playwright_cli.sh`
  - `window.__AW_TEST__`（`runSteps/setMode/focus/select/sendControl/getState`）
- Edge Cases & Error Handling:
  - F1 `ERR_CONNECTION_REFUSED`: launcher 未就绪或已退出；先确认端口监听再重试。
  - F2 渲染初始化崩溃（如 `RuntimeError: unreachable`、`CONTEXT_LOST_WEBGL`）：立即归档证据并标记失败。
  - F3 `connecting + tick=0` 长时间不推进：先执行 `play` 并额外观察约 12 秒，仍无推进则失败。
  - F4 URL 在 `source` 场景解析失败：强制使用带引号 URL，避免 `&` 被 shell 截断。
  - 会话残留：每轮前 `close-all`，降低 daemon/session 干扰。
  - 视觉门禁假通过：full coverage 需额外校验 `capture_status.txt` 的 `connection_status=connected` 与 `snapshot_ready=1`。
- Non-Functional Requirements:
  - NFR-WEB-1: 首轮 Web smoke 在环境就绪后 5 分钟内完成首个 verdict。
  - NFR-WEB-2: 证据产物路径固定在 `output/playwright/`，可供自动审阅。
  - NFR-WEB-3: 门禁误报率可控，必须通过 fail-fast 分类输出原因。
  - NFR-WEB-4: 关键脚本参数/命令口径在主手册与分册中保持一致。
- Security & Privacy: 采样日志与截图不得包含凭据，控制台输出仅保留问题定位所需信息。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (WPCM-1): 从 `testing-manual.md` 拆分 Web Playwright 分册并建立唯一入口。
  - v1.1 (WPCM-2): 增加启动前自检、会话防抖与 F1~F4 fail-fast 处置。
  - v2.0 (WPCM-3): 强化 GPU + headed 硬门禁与软件渲染阻断。
  - v2.1 (WPCM-4): 对齐一键 QA/full coverage 产物门禁与失败策略。
  - v2.2 (WPCM-5): strict schema 人工迁移与命名统一收口。
- Technical Risks:
  - 风险-1: 环境波动导致端口/连接不稳定，引发假失败。
  - 风险-2: 软件渲染误入验收口径，导致性能与视觉结论失真。
  - 风险-3: 脚本与手册偏离，造成执行路径不一致。
  - 风险-4: 仅凭截图判定导致“有图但不可用”漏检。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-TESTING-WEB-001 | WPCM-1/2/5 | `test_tier_required` | 启动与采样命令审阅 + 文档治理检查 | Web smoke 执行稳定性 |
| PRD-TESTING-WEB-002 | WPCM-2/3/4 | `test_tier_required` | GPU/headed 门禁与最小通过标准核验 | 发布验收真实性 |
| PRD-TESTING-WEB-003 | WPCM-2/4/5 | `test_tier_required` | F1~F4 分级处置流程抽样 + 脚本产物校验 | 故障处置与审计追溯 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-WEB-001 | Web 闭环默认链路，Playwright 优先 | 继续以 native 抓图为默认 | Web 链路更贴近发布场景且可自动化。 |
| DEC-WEB-002 | 验收强制 headed + GPU | 允许 headless 或软件渲染 | 避免性能/视觉口径失真。 |
| DEC-WEB-003 | 语义化 `__AW_TEST__` 操作优先 | 纯坐标点击脚本 | 减少 UI 变动导致的脆弱性。 |
| DEC-WEB-004 | 失败分级 F1~F4 + 证据归档 | 仅记录通用失败日志 | 缩短定位时间并提升复盘质量。 |
| DEC-WEB-005 | legacy 文档逐篇人工迁移 | 脚本批量改写 | 保证历史约束和执行语义完整。 |

## 原文约束点映射（内容保真）
- 原“目标：统一 Web 闭环启动、采样、门禁、排障” -> 第 1 章 Problem/Solution/SC。
- 原“S6 启动命令、自检、GPU+headed、采样步骤与会话防抖” -> 第 2 章流程/规格矩阵 + 第 4 章技术规格。
- 原“最小通过标准（canvas、`__AW_TEST__`、console、截图）” -> 第 2 章 AC。
- 原“Fail Fast F1~F4 与处置” -> 第 4 章 Edge Cases & Error Handling。
- 原“一键发行验收与 full coverage 门禁” -> 第 2 章 Flow-WEB-005 + 第 5 章 roadmap。
- 原“Web 默认链路、native fallback、语义操作约定、调试收尾建议” -> 第 4 章架构/集成与 NFR。
