> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-20

# Agent World Runtime：Node 主循环基础模块

## 1. Executive Summary
- Problem Statement: 新增独立 `node` crate，提供可复用的节点主循环基础能力，供 `agent_world` 在启动模拟时直接调用。
- Proposed Solution: 将“节点生命周期控制”从业务入口中抽离，统一为 `NodeRuntime` 接口（启动、停止、状态快照）。
- Success Criteria:
  - SC-1: 先落地最小闭环：单进程可启动一个逻辑节点并周期运行 tick，为后续接入网络/共识执行路径预留扩展点。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：Node 主循环基础模块 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 新增 `crates/agent_world_node`，包含：
  - AC-2: `NodeRole`：节点角色枚举（`sequencer` / `storage` / `observer`）。
  - AC-3: `NodeConfig`：节点配置（`node_id` / `world_id` / `tick_interval` / `role`）。
  - AC-4: `NodeRuntime`：节点主循环生命周期接口（`start` / `stop` / `snapshot`）。
  - AC-5: `NodeSnapshot`：运行态快照（是否运行、tick 计数、最后 tick 时间）。
  - AC-6: 在 `world_viewer_live` 启动流程中默认启动一个节点实例，并支持 CLI 配置。
- Non-Goals:
  - 多节点真实网络同步与跨进程消息分发。
  - 共识提案/投票执行与区块提交闭环。
  - 持久化节点状态恢复（重启续跑）。
  - 运维面板与可视化接线。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-node-mainloop.prd.md`
  - `doc/p2p/archive/distributed-node-mainloop.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### NodeRole
- `Sequencer`
- `Storage`
- `Observer`

### NodeConfig
- `node_id: String`
- `world_id: String`
- `tick_interval: Duration`
- `role: NodeRole`

### NodeRuntime
- `new(config) -> NodeRuntime`
- `start() -> Result<(), NodeError>`
- `stop() -> Result<(), NodeError>`
- `snapshot() -> NodeSnapshot`

### NodeSnapshot
- `running: bool`
- `tick_count: u64`
- `last_tick_unix_ms: Option<i64>`

## 5. Risks & Roadmap
- Phased Rollout:
  - DNM-1：设计文档与项目管理文档落地。
  - DNM-2：`node` crate 基础类型与主循环实现。
  - DNM-3：`agent_world` 启动模拟时接入节点自动启动。
  - DNM-4：测试回归、文档状态收口与 devlog。
- Technical Risks:
  - 当前主循环仅提供最小 tick 能力，未承载真实网络/共识逻辑，后续需要平滑扩展接口。
  - 若入口默认启动节点，需注意未来多角色组合时的参数复杂度与可观测性。
  - 节点运行线程与主服务线程并行，需确保停止流程可靠，避免测试残留线程。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-040-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-040-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
