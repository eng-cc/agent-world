# world-simulator PRD

## 目标
- 建立 world-simulator 模块设计主文档，统一需求边界、技术方案与验收标准。
- 确保 world-simulator 模块后续改动可追溯到 PRD-ID、任务和测试。
- 为专题能力提供分册挂载机制，保持主 PRD 可导航、可审计。

## 范围
- 覆盖 world-simulator 模块当前能力设计、接口边界、测试口径与演进路线。
- 覆盖 PRD-ID 到 `doc/world-simulator/prd.project.md` 的任务映射。
- 覆盖启动器链路中的链上转账能力（通过分册维护详细条款）。
- 不覆盖实现代码逐行说明与历史过程记录。

## 接口 / 数据
- PRD 主入口: `doc/world-simulator/prd.md`
- 项目管理入口: `doc/world-simulator/prd.project.md`
- 追踪主键: `PRD-WORLD_SIMULATOR-xxx`
- 测试与发布参考: `testing-manual.md`
- 分册索引:
  - `doc/world-simulator/prd/acceptance/unified-checklist.md`（PRD-WORLD_SIMULATOR-001/002）
  - `doc/world-simulator/prd/launcher/blockchain-transfer.md`（PRD-WORLD_SIMULATOR-004/005）

## 里程碑
- M1 (2026-03-03): 完成模块设计 PRD 主体重写与任务改造。
- M2 (2026-03-03): 完成启动器链上转账需求建模与任务拆解。
- M3: 完成 PRD 分册结构落地，主入口仅保留总览与导航。
- M4: 完成链运行时转账 API、运行时账本动作、启动器交互与闭环验收。

## 风险
- 模块边界演进快，文档同步可能滞后。
- 指标口径不稳定会降低验收一致性。
- 分册与主入口不同步会导致需求追踪断裂。
## 1. Executive Summary
- Problem Statement: world-simulator 涵盖场景、viewer、LLM 与 launcher 多条链路，需求持续增长时若全部堆叠在单文档，会降低可维护性并影响变更追踪效率。
- Proposed Solution: 主 PRD 保持模块级边界与验收总口径，专题能力迁移到分册；启动器链上转账作为首个分册能力，维护完整细节。
- Success Criteria:
  - SC-1: simulator/viewer/launcher 变更全部映射 PRD-WORLD_SIMULATOR-ID。
  - SC-2: Web 闭环路径作为默认链路并保持可复现测试证据。
  - SC-3: 场景系统关键基线（初始化、资源、交互）具备稳定回归。
  - SC-4: LLM 交互链路具备可观察性与降级策略记录。
  - SC-5: 启动器链上转账需求在分册中完整定义，并与主 PRD / 项目任务保持一一映射。
  - SC-6: 分册变更后主 PRD 仍可作为唯一入口完成导航与验收追踪。
  - SC-7: 场景系统、Viewer、启动器验收口径统一到同一 checklist，并可直接映射到测试证据。

## 2. User Experience & Functionality
- User Personas:
  - 模拟层开发者：需要统一场景与运行语义。
  - Viewer 体验开发者：需要明确 Web/Native 行为边界与验收标准。
  - 发布与测试人员：需要可执行的闭环测试与证据产物。
  - 启动器玩家：需要在同一入口内完成资产查询与转账，无需命令行。
- User Stories:
  - PRD-WORLD_SIMULATOR-001: As a 模拟层开发者, I want unified world-simulator contracts, so that scenario evolution is stable.
  - PRD-WORLD_SIMULATOR-002: As a Viewer 开发者, I want consistent web-first UX rules, so that user paths remain predictable.
  - PRD-WORLD_SIMULATOR-003: As a 发布人员, I want reproducible simulator closure tests, so that releases are verifiable.
  - PRD-WORLD_SIMULATOR-004: As a 启动器玩家, I want to submit a blockchain transfer in launcher, so that I can move main token balances without external tools.（详见分册）
  - PRD-WORLD_SIMULATOR-005: As a 链路开发者, I want transfer requests to be replay-safe and traceable, so that transfer execution is secure and auditable.（详见分册）
- Acceptance Criteria:
  - AC-1: world-simulator PRD 覆盖场景、Viewer、LLM、启动器四条主线。
  - AC-2: world-simulator project 文档维护任务拆解与状态。
  - AC-3: 与 `doc/world-simulator/scenario/scenario-files.md`、`doc/world-simulator/viewer/viewer-web-closure-testing-policy.md` 等专题文档一致。
  - AC-4: 关键交互变更同步更新 testing 手册与测试记录。
  - AC-5: 分册内专题条款（接口/安全/测试）在主 PRD 中可定位、在项目文档中可执行。
  - AC-6: 统一验收清单覆盖场景、Viewer Web 闭环、启动器入口与证据模板，并与 `testing-manual.md` 一致。
- Non-Goals:
  - 不在本 PRD 中详细列出每个 UI 像素级规范。
  - 不替代 world-runtime/p2p 的底层协议设计。
  - 不在主 PRD 中展开专题实现细节（转账细节迁移至分册）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: LLM 回归脚本、Playwright 闭环、场景矩阵测试、启动器集成测试。
- Evaluation Strategy: 以场景启动成功率、关键交互完成率、LLM 链路稳定性、闭环缺陷收敛时间评估。

## 4. Technical Specifications
- Architecture Overview: world-simulator 连接 runtime 与 viewer，负责把世界状态转化为可交互体验，并通过场景系统与启动器提供可复现实验环境。专题能力通过分册文档按域维护。
- Integration Points:
  - `doc/world-simulator/scenario/scenario-files.md`
  - `doc/world-simulator/viewer/viewer-web-closure-testing-policy.md`
  - `doc/world-simulator/launcher/game-unified-launcher-2026-02-27.md`
  - `doc/world-simulator/prd/acceptance/unified-checklist.md`
  - `doc/world-simulator/prd/launcher/blockchain-transfer.md`
  - `testing-manual.md`
- Security & Privacy:
  - Viewer/launcher 链路涉及配置与鉴权注入时必须最小暴露。
  - 调试接口需受限并可审计。
  - 专题安全规则在对应分册中维护（含转账安全约束）。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (2026-03-03): 固化 world-simulator 主 PRD 边界。
  - v1.1: 完成分册化治理与启动器链上转账专题建模。
  - v2.0: 建立体验质量趋势指标（启动、交互、性能、稳定性）。
- Technical Risks:
  - 风险-1: 前端体验迭代快导致行为回归频发。
  - 风险-2: LLM 外部依赖波动影响端到端稳定性。
  - 风险-3: 分册索引维护缺失导致需求追踪断链。
