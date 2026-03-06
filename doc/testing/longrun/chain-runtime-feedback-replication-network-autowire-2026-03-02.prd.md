# Agent World: Chain Runtime 反馈网络复制层自动挂载修复（2026-03-02）

审计轮次: 3

## 1. Executive Summary
- Problem Statement: `world_chain_runtime` 默认开启 `feedback_p2p` 时，若未预先挂载 replication network，会在启动阶段直接报错 `InvalidConfig("feedback_p2p requires replication network")`，导致单机链路不可用。
- Proposed Solution: 在 `world_chain_runtime` 内部提供默认 replication network 自动挂载，并将启动时序调整为“先挂载 network handle，再执行 runtime.start()”，同时启用无 peers 的本地 fallback。
- Success Criteria:
  - SC-1: 默认配置下 `world_chain_runtime` 可成功启动，不再因缺少 replication network 中断。
  - SC-2: 默认挂载使用 loopback + ephemeral 端口（`/ip4/127.0.0.1/tcp/0`），不破坏现有 CLI/HTTP 接口。
  - SC-3: `allow_local_handler_fallback_when_no_peers=true` 生效，单机可完成 feedback announce/fetch 闭环。
  - SC-4: 定向测试覆盖默认网络配置与启动烟测路径，结果可复现。

## 2. User Experience & Functionality
- User Personas:
  - 启动器/链路开发者：需要默认可启动，不希望被配置前置条件卡死。
  - 测试维护者：需要单机 longrun 与回归链路稳定可跑。
  - 发布负责人：需要在不扩展外部参数的前提下修复启动阻塞缺陷。
- User Scenarios & Frequency:
  - 本地链运行时启动：每次本地调试与回归都会触发。
  - 长跑脚本预热：每次执行 longrun 前置启动流程触发。
  - 故障复盘复测：遇到 `feedback_p2p` 启动失败时触发。
- User Stories:
  - PRD-TESTING-LONGRUN-AUTONET-001: As a 链路开发者, I want runtime to auto-mount a default replication network, so that feedback_p2p does not fail at startup.
  - PRD-TESTING-LONGRUN-AUTONET-002: As a 测试维护者, I want no-peer local fallback enabled by default, so that single-node feedback loops remain testable.
  - PRD-TESTING-LONGRUN-AUTONET-003: As a 发布负责人, I want startup ordering and config behavior to be deterministic and tested, so that rollout risk stays bounded.
- Critical User Flows:
  1. Flow-AUTONET-001: `NodeRuntime::new(config) -> 构建默认 Libp2pReplicationNetworkConfig -> 挂载 NodeReplicationNetworkHandle -> runtime.start()`
  2. Flow-AUTONET-002: `无 peers 场景 -> 本地 handler fallback 生效 -> feedback announce/fetch 完成闭环`
  3. Flow-AUTONET-003: `执行定向测试与烟测 -> 校验 ready 日志 -> 文档与项目状态收口`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 默认 replication network 自动挂载 | listen=`/ip4/127.0.0.1/tcp/0`、`Libp2pReplicationNetworkConfig` | runtime 初始化后自动构建并挂载 network handle | `runtime-new -> network-mounted -> runtime-started` | 优先保障默认可启动，不改现有 CLI 覆盖入口 | 运行时维护者可修改默认参数 |
| 无 peers 本地 fallback | `allow_local_handler_fallback_when_no_peers=true` | peers 为空时允许本地 handler 处理 feedback 流程 | `no-peers -> local-fallback-active` | 仅在无 peers 场景兜底，不替代多节点联通验证 | 测试维护者可在回归中验证 |
| 启动时序修复 | `NodeRuntime::new` 与 `runtime.start` 中间挂载句柄 | 启动前完成 network 依赖注入 | `dependency-missing -> dependency-ready` | 固定顺序，避免 nondeterministic 初始化失败 | 核心 runtime 路径受控 |
| 回归验证收口 | 单测、check、启动烟测命令结果 | 执行并记录验证证据 | `planned -> passed/failed` | 先单测/静态检查，再烟测 | QA/维护者审阅 |
- Acceptance Criteria:
  - AC-1: `feedback_p2p` 默认启用时，`world_chain_runtime` 不再出现 `requires replication network` 启动错误。
  - AC-2: 默认 network 配置使用 loopback + ephemeral 监听地址并有测试覆盖。
  - AC-3: 无 peer 场景可完成本地 fallback，不阻塞 feedback 闭环。
  - AC-4: 启动顺序修复不引入 CLI 参数和 HTTP 接口变更。
  - AC-5: 任务文档与 devlog 记录完整，满足 PRD-ID 追踪。
- Non-Goals:
  - 不在本任务实现多节点拓扑参数化。
  - 不修改 feedback 协议语义或 announce/fetch 数据结构。
  - 不新增发布门禁指标。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题是链运行时网络依赖修复，不涉及 AI 推理模型或评测链路变更）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 通过在 `world_chain_runtime` 启动阶段注入默认 replication network handle，修复 `feedback_p2p` 对 network 依赖未满足导致的启动失败，并保持外部接口稳定。
- Integration Points:
  - `crates/agent_world/src/bin/world_chain_runtime.rs`
  - `crates/agent_world/src/bin/world_chain_runtime/world_chain_runtime_tests.rs`
  - `doc/testing/longrun/chain-runtime-feedback-replication-network-autowire-2026-03-02.prd.project.md`
  - `doc/devlog/2026-03-02.md`
- Edge Cases & Error Handling:
  - 无 peers 网络：启用本地 fallback 保证单机闭环可用。
  - 仅 loopback 监听：跨节点联通不在本修复范围，需后续参数化专题处理。
  - 启动顺序回归：若后续重构破坏“先挂载后启动”，测试应即时失败。
  - 配置注入失败：应输出可定位错误，禁止静默降级。
- Non-Functional Requirements:
  - NFR-AUTONET-1: 默认启动路径稳定可复现，单机场景启动成功率 100%。
  - NFR-AUTONET-2: 修复不引入新增 CLI/HTTP 配置复杂度。
  - NFR-AUTONET-3: 回归测试在 `test_tier_required` 可稳定执行。
  - NFR-AUTONET-4: 文档与实现追踪链路完整（PRD-ID -> Task -> Test）。
- Security & Privacy: 默认 loopback 监听降低暴露面；跨主机部署需后续专题补充网络安全策略。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (AUTONET-1): 建立专题设计与项目管理文档。
  - v1.1 (AUTONET-2): runtime 启动前自动挂载默认 replication network 并启用 no-peer fallback。
  - v2.0 (AUTONET-3): 补齐测试与启动烟测，收口文档状态。
  - v2.1 (AUTONET-4): 按 strict schema 完成文档迁移与命名切换。
- Technical Risks:
  - 风险-1: loopback + ephemeral 仅覆盖单机闭环，易被误解为跨节点可用。
  - 风险-2: no-peer fallback 可能掩盖真实拓扑配置缺失。
  - 风险-3: 后续 runtime 初始化重构可能再次引入依赖顺序回归。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-TESTING-LONGRUN-AUTONET-001 | AUTONET-1/2 | `test_tier_required` | 启动路径验证不再报 `requires replication network` | chain runtime 启动稳定性 |
| PRD-TESTING-LONGRUN-AUTONET-002 | AUTONET-2/3 | `test_tier_required` | 默认 network 配置单测 + no-peer fallback 路径验证 | feedback 本地闭环可用性 |
| PRD-TESTING-LONGRUN-AUTONET-003 | AUTONET-3/4 | `test_tier_required` | `cargo test/check` + `cargo run` 启动烟测 + 文档治理检查 | longrun 启动链路与文档追踪一致性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-AUTONET-001 | 在 runtime 内部自动挂载默认 replication network | 继续要求外部手工配置 network handle | 先修复默认启动阻塞，降低调试与测试成本。 |
| DEC-AUTONET-002 | 默认启用 no-peer 本地 fallback | 无 peers 直接失败 | 单机闭环是当前 longrun 与回归的基本前提。 |
| DEC-AUTONET-003 | 修复限定在启动依赖与默认配置，不扩展 CLI | 同步引入跨节点参数化改造 | 控制变更范围，避免一次任务引入过多不确定性。 |

## 原文约束点映射（内容保真）
- 原“目标（修复 feedback_p2p 依赖 replication network 导致启动失败）” -> 第 1 章 Problem/Solution/SC。
- 原“范围（runtime/tests/项目文档/devlog）” -> 第 4 章 Integration Points。
- 原“接口/数据（默认 listen、Libp2p 配置、fallback、启动顺序）” -> 第 2 章规格矩阵 + 第 4 章技术规格。
- 原“里程碑 M1~M4” -> 第 5 章 Phased Rollout（AUTONET-1~4）。
- 原“风险（单机闭环边界、fallback 掩盖问题）” -> 第 4 章 Edge Cases + 第 5 章 Technical Risks。
- 原“完成态与验证命令” -> 第 6 章 Test Plan & Traceability。
