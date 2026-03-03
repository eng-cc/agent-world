# core PRD

## 目标
- 作为项目级总 PRD，提供 Agent World 的全局设计全貌入口。
- 统一跨模块边界、关键链路、术语口径与验收基线。
- 确保各模块改动可追溯到 PRD-ID、任务和测试证据。

## 范围
- 覆盖全项目模块地图、端到端链路、关键分册导航与治理基线。
- 覆盖 PRD-ID 到 `doc/core/prd.project.md` 的任务映射。
- 不覆盖各模块实现细节正文（由模块 PRD 与专题分册承载）。

## 接口 / 数据
- 项目级 PRD 入口: `doc/core/prd.md`
- 项目管理入口: `doc/core/prd.project.md`
- 追踪主键: `PRD-CORE-xxx`
- 模块入口总览: `doc/README.md`
- 测试与发布参考: `testing-manual.md`

## 里程碑
- M1 (2026-03-03): 完成模块 PRD 体系重构并建立项目级总览入口。
- M2: 固化跨模块变更影响检查清单（设计/代码/测试/发布）。
- M3: 建立 PRD-ID -> Task -> Test 追踪报表。

## 风险
- 模块并行演进过快时，全局总览可能滞后于真实实现。
- 模块间术语不统一会造成评审误判与接口漂移。

## 1. Executive Summary
- Problem Statement: 项目已拆分为多个模块 PRD，但缺少一个“只读一份文档即可掌握整体设计”的全局总入口。
- Proposed Solution: 在 core PRD 中固化项目全局模块地图、关键端到端链路、关键分册导航和统一治理口径，使其成为仓库级设计总览。
- Success Criteria:
  - SC-1: `doc/README.md` 将 `doc/core/prd.md` 作为推荐阅读第一入口。
  - SC-2: core PRD 明确列出全部模块职责、关键链路与关键分册。
  - SC-3: 跨模块改动评审可基于 core PRD 完成影响面识别。
  - SC-4: 新增模块级需求可映射到对应模块 PRD 与 core 基线。

## 2. User Experience & Functionality
- User Personas:
  - 架构负责人：需要一份文档快速把握全局设计与边界。
  - 模块维护者：需要明确自己模块在全局链路中的位置与依赖。
  - 发布负责人：需要统一口径判定跨模块风险与放行条件。
- User Stories:
  - PRD-CORE-001: As an 架构负责人, I want a project-wide blueprint, so that I can reason about cross-module impact quickly.
  - PRD-CORE-002: As a 模块维护者, I want one place to see end-to-end chains, so that I can design compatible changes.
  - PRD-CORE-003: As a 发布负责人, I want unified release/test governance, so that go/no-go decisions are auditable.
- Acceptance Criteria:
  - AC-1: core PRD 包含模块职责矩阵。
  - AC-2: core PRD 包含至少 4 条关键端到端链路描述。
  - AC-3: core PRD 给出关键分册导航并可从 `doc/README.md` 到达。
  - AC-4: core project 文档任务与 PRD-CORE-ID 可映射。
- Non-Goals:
  - 不在 core PRD 中替代模块详细技术分册。
  - 不在 core PRD 中维护逐版本实现变更流水（该信息在 devlog）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 文档治理检查脚本、`rg` 检索、测试手册与 CI 脚本用于核验跨模块一致性。
- Evaluation Strategy: 以跨模块评审返工率、口径冲突数、发布前补文档次数评估全局 PRD 有效性。

## 4. Technical Specifications
- Architecture Overview: core 作为“全局设计总览层”，不承载业务实现代码，而承载全局结构、统一约束和跨模块链路描述。

### 项目模块地图（Design Map）
| 模块 | 主职责 | 关键实现载体 |
| --- | --- | --- |
| core | 全局设计总览、跨模块治理基线 | `doc/core/*` |
| engineering | 工程规范、文件约束、质量门禁 | `doc/engineering/*`, `scripts/*`, CI workflows |
| game | 玩法循环、治理/经济/战争规则设计 | `doc/game/*`, `crates/agent_world` (gameplay相关) |
| world-runtime | 世界内核、事件溯源、WASM执行与治理 | `doc/world-runtime/*`, `crates/agent_world`, `crates/agent_world_wasm_*` |
| world-simulator | 场景系统、Viewer/Launcher、LLM交互链路 | `doc/world-simulator/*`, `crates/agent_world_viewer`, `crates/agent_world_client_launcher` |
| p2p | 网络、共识、DistFS、多节点运行 | `doc/p2p/*`, `crates/agent_world_net`, `crates/agent_world_consensus`, `crates/agent_world_distfs`, `crates/agent_world_node` |
| headless-runtime | 无界面运行链路、鉴权、长稳运维能力 | `doc/headless-runtime/*`, `crates/agent_world/src/bin/*` |
| testing | 分层测试体系与发布门禁 | `doc/testing/*`, `testing-manual.md`, `scripts/ci-tests.sh` |
| scripts | 自动化脚本能力与执行约束 | `scripts/*`, `doc/scripts/*` |
| site | 站点信息架构、发布内容、SEO | `site/*`, `doc/site/*` |
| readme | 对外文档入口口径统一 | `README.md`, `doc/readme/*` |
| playability_test_result | 可玩性反馈证据与发布引用 | `doc/playability_test_result/*`, `doc/playability_test_result/game-test.md` |

### 关键端到端链路（E2E Chains）
1. 玩家交互链路:
`Launcher/Viewer -> world_viewer_live/world_chain_runtime -> world-runtime -> event/journal -> UI反馈`
2. 世界执行链路:
`Action/Intent -> Rule Validation -> Resource/State Transition -> Event -> Snapshot/Replay`
3. 模块扩展链路:
`Rust Source -> WASM Artifact -> Register/Install -> Runtime Sandbox Execution -> Governance/Audit`
4. 分布式一致性链路:
`Node/Net -> Consensus Commit -> DistFS/State Replication -> Runtime Apply -> Viewer Observe`
5. 发布验证链路:
`PRD-ID Task -> test_tier_required/full -> Web闭环/长跑 -> Evidence Bundle -> Release Decision`

### 关键分册导航（只读总览后优先下钻）
- 运行时内核: `doc/world-runtime/runtime/runtime-integration.md`
- WASM 接口与执行: `doc/world-runtime/wasm/wasm-interface.md`, `doc/world-runtime/wasm/wasm-executor.md`
- 场景矩阵: `doc/world-simulator/scenario/scenario-files.md`
- Web 闭环测试策略: `doc/world-simulator/viewer/viewer-web-closure-testing-policy.md`
- 分布式路线图: `doc/p2p/blockchain/production-grade-blockchain-p2pfs-roadmap.md`
- 系统性测试手册: `testing-manual.md`

### 全局术语（Glossary）
- PRD-ID: 需求追踪主键，连接 PRD、任务、测试与发布证据。
- required/full: 分层测试的两级核心门禁。
- Web-first 闭环: 默认 UI 验证路径（Playwright 优先）。
- Effect/Receipt: 运行时外部副作用与回执审计机制。
- Snapshot/Replay: 世界状态持久化与可重放能力。

- Integration Points:
  - `AGENTS.md`
  - `doc/README.md`
  - `testing-manual.md`
  - 各模块 `doc/<module>/prd.md` 与 `doc/<module>/prd.project.md`
- Security & Privacy: core 仅维护结构与治理口径；涉及密钥、签名、隐私数据的要求由对应模块 PRD 细化并执行。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (2026-03-03): core PRD 成为项目级总览入口。
  - v1.1: 建立跨模块变更影响检查清单模板。
  - v2.0: 建立 PRD-ID 到测试证据的自动化追踪报表。
- Technical Risks:
  - 风险-1: 模块新增能力未及时回填全局链路。
  - 风险-2: 总览与分册的口径同步依赖人工流程。
