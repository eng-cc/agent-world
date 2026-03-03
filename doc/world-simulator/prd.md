# world-simulator PRD

## 目标
- 建立 world-simulator 模块设计主文档，统一需求边界、技术方案与验收标准。
- 确保 world-simulator 模块后续改动可追溯到 PRD-ID、任务和测试。

## 范围
- 覆盖 world-simulator 模块当前能力设计、接口边界、测试口径与演进路线。
- 覆盖 PRD-ID 到 `doc/world-simulator/prd.project.md` 的任务映射。
- 不覆盖实现代码逐行说明与历史过程记录。

## 接口 / 数据
- PRD 主入口: `doc/world-simulator/prd.md`
- 项目管理入口: `doc/world-simulator/prd.project.md`
- 追踪主键: `PRD-WORLD_SIMULATOR-xxx`
- 测试与发布参考: `testing-manual.md`

## 里程碑
- M1 (2026-03-03): 完成模块设计 PRD 主体重写与任务改造。
- M2: 补齐模块设计验收清单与关键指标。
- M3: 建立 PRD-ID -> Task -> Test 的长期追踪闭环。

## 风险
- 模块边界演进快，文档同步可能滞后。
- 指标口径不稳定会降低验收一致性。
## 1. Executive Summary
- Problem Statement: world-simulator 同时承载世界初始化、场景系统、Viewer Web/Native、LLM 交互与启动器链路，缺少统一模块 PRD 时容易出现体验层与运行层口径分叉。
- Proposed Solution: 以 world-simulator PRD 统一定义模拟层目标、Viewer 交互能力、LLM 接入边界、场景验证标准与发布质量目标。
- Success Criteria:
  - SC-1: simulator/viewer/launcher 变更全部映射 PRD-WORLD_SIMULATOR-ID。
  - SC-2: Web 闭环路径作为默认链路并保持可复现测试证据。
  - SC-3: 场景系统关键基线（初始化、资源、交互）具备稳定回归。
  - SC-4: LLM 交互链路具备可观察性与降级策略记录。

## 2. User Experience & Functionality
- User Personas:
  - 模拟层开发者：需要统一场景与运行语义。
  - Viewer 体验开发者：需要明确 Web/Native 行为边界与验收标准。
  - 发布与测试人员：需要可执行的闭环测试与证据产物。
- User Stories:
  - PRD-WORLD_SIMULATOR-001: As a 模拟层开发者, I want unified world-simulator contracts, so that scenario evolution is stable.
  - PRD-WORLD_SIMULATOR-002: As a Viewer 开发者, I want consistent web-first UX rules, so that user paths remain predictable.
  - PRD-WORLD_SIMULATOR-003: As a 发布人员, I want reproducible simulator closure tests, so that releases are verifiable.
- Acceptance Criteria:
  - AC-1: world-simulator PRD 覆盖场景、Viewer、LLM、启动器四条主线。
  - AC-2: world-simulator project 文档维护任务拆解与状态。
  - AC-3: 与 `doc/world-simulator/scenario/scenario-files.md`、`doc/world-simulator/viewer/viewer-web-closure-testing-policy.md` 等专题文档一致。
  - AC-4: 关键交互变更同步更新 testing 手册与测试记录。
- Non-Goals:
  - 不在本 PRD 中详细列出每个 UI 像素级规范。
  - 不替代 world-runtime/p2p 的底层协议设计。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: LLM 回归脚本、Playwright 闭环、场景矩阵测试、启动器集成测试。
- Evaluation Strategy: 以场景启动成功率、关键交互完成率、LLM 链路稳定性、闭环缺陷收敛时间评估。

## 4. Technical Specifications
- Architecture Overview: world-simulator 连接 runtime 与 viewer，负责把世界状态转化为可交互体验，并通过场景系统与启动器提供可复现实验环境。
- Integration Points:
  - `doc/world-simulator/scenario/scenario-files.md`
  - `doc/world-simulator/viewer/viewer-web-closure-testing-policy.md`
  - `doc/world-simulator/launcher/game-unified-launcher-2026-02-27.md`
  - `testing-manual.md`
- Security & Privacy: Viewer/launcher 链路涉及配置与鉴权注入时必须最小暴露；调试接口需受限并可审计。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (2026-03-03): 固化 world-simulator 统一设计边界。
  - v1.1: 补齐 Web-first 闭环与场景矩阵的版本化验收。
  - v2.0: 建立体验质量趋势指标（启动、交互、性能、稳定性）。
- Technical Risks:
  - 风险-1: 前端体验迭代快导致行为回归频发。
  - 风险-2: LLM 外部依赖波动影响端到端稳定性。
