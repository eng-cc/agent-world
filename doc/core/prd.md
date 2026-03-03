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
- 文件级索引: doc/core/prd.index.md
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
- User Scenarios & Frequency:
  - 架构评审：每次跨模块需求评审前至少 1 次，核对影响边界与依赖。
  - 模块联调：每周多次，按链路检查上游/下游耦合点是否一致。
  - 发布评估：每个版本候选至少 1 次，基于统一门禁做 go/no-go 判定。
  - 新成员入项：入项首日使用，快速建立项目全局认知。
- User Stories:
  - PRD-CORE-001: As an 架构负责人, I want a project-wide blueprint, so that I can reason about cross-module impact quickly.
  - PRD-CORE-002: As a 模块维护者, I want one place to see end-to-end chains, so that I can design compatible changes.
  - PRD-CORE-003: As a 发布负责人, I want unified release/test governance, so that go/no-go decisions are auditable.
- Critical User Flows:
  1. Flow-CORE-001: `读取模块地图 -> 识别改动所属模块 -> 定位上下游依赖 -> 形成影响面清单`
  2. Flow-CORE-002: `读取关键链路 -> 映射到模块 PRD-ID -> 对照测试分层 -> 输出发布风险判断`
  3. Flow-CORE-003: `发现口径冲突 -> 回溯分册来源 -> 在 core 基线中统一术语与边界 -> 回写模块文档`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 模块地图导航 | 模块名、职责、关键载体、入口路径 | 进入模块 PRD 与 project 文档 | `draft -> reviewed -> published` | 默认按模块分层顺序展示 | 所有贡献者可读，维护者可改 |
| 关键链路追踪 | 链路名称、上游、下游、测试门禁 | 依据链路定位依赖变更与测试范围 | `identified -> validated -> archived` | 高风险链路优先检查 | 发布负责人具备最终裁定权 |
| 术语与口径统一 | 术语名、定义、引用文档、更新时间 | 发现冲突后统一定义并回写引用 | `conflict -> resolved -> synced` | 以核心术语集为唯一优先级 | core 维护者审核后生效 |
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
| playability_test_result | 可玩性反馈证据与发布引用 | `doc/playability_test_result/*`, `doc/playability_test_result/game-test.prd.md` |

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
- WASM 接口与执行: `doc/world-runtime/wasm/wasm-interface.md`, `doc/world-runtime/wasm/wasm-executor.prd.md`
- 场景矩阵: `doc/world-simulator/scenario/scenario-files.prd.md`
- Web 闭环测试策略: `doc/world-simulator/viewer/viewer-web-closure-testing-policy.prd.md`
- 分布式路线图: `doc/p2p/blockchain/production-grade-blockchain-p2pfs-roadmap.prd.md`
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
- Edge Cases & Error Handling:
  - 模块入口失效：若目标路径迁移，core 必须同步更新导航并保留可追溯说明。
  - 信息缺失：若模块 PRD 尚未更新，标记“口径待同步”并阻断发布结论。
  - 版本漂移：core 与分册冲突时，以最近审阅通过版本为准并触发修复任务。
  - 依赖冲突：同一链路被多个模块修改时，需合并影响面并重跑 required 级验证。
  - 测试证据缺口：无证据不得判定链路通过，必须补齐最小 required 证据。
  - 术语冲突：同术语多定义时优先使用 core 词典并登记决策记录。
- Non-Functional Requirements:
  - NFR-CORE-1: 核心入口文档链接可用率 100%。
  - NFR-CORE-2: 跨模块评审时，影响面识别耗时 <= 30 分钟。
  - NFR-CORE-3: 发布评审前，PRD-ID 到测试证据映射完整率 100%。
  - NFR-CORE-4: 所有核心术语变更需在 1 个工作日内同步到相关入口文档。
  - NFR-CORE-5: core 主文档维持 <= 500 行，超限必须拆分分册。
- Security & Privacy: core 仅维护结构与治理口径；涉及密钥、签名、隐私数据的要求由对应模块 PRD 细化并执行。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (2026-03-03): core PRD 成为项目级总览入口。
  - v1.1: 建立跨模块变更影响检查清单模板。
  - v2.0: 建立 PRD-ID 到测试证据的自动化追踪报表。
- Technical Risks:
  - 风险-1: 模块新增能力未及时回填全局链路。
  - 风险-2: 总览与分册的口径同步依赖人工流程。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-CORE-001 | TASK-CORE-001/002/006/007 | `test_tier_required` | 入口完整性扫描、模块地图与导航可达检查 | 全项目入口与架构总览一致性 |
| PRD-CORE-002 | TASK-CORE-002/003/004/007 | `test_tier_required` + `test_tier_full` | 关键链路映射核验、跨模块依赖抽样复核 | 跨模块设计兼容性与发布评审效率 |
| PRD-CORE-003 | TASK-CORE-004/005/007 | `test_tier_required` + `test_tier_full` | 发布门禁证据映射校验、季度一致性审查记录检查 | 发布决策可审计性与长期治理稳定性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-CORE-001 | 将 core 固化为项目全局唯一总览入口 | 各模块独立维护无全局入口 | 降低跨模块认知成本并提升评审效率。 |
| DEC-CORE-002 | 使用 PRD-ID 作为跨文档追踪主键 | 使用任务编号作为唯一主键 | PRD-ID 可跨任务周期稳定复用并支持审计。 |
| DEC-CORE-003 | 发布评审需绑定 required/full 测试证据 | 仅凭人工结论放行 | 可追溯性与一致性显著更高。 |
