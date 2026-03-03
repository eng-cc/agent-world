> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-20

# Agent World Runtime：Node PoS 主循环接入

## 1. Executive Summary
- Problem Statement: 将现有 `node` 基础主循环升级为可驱动“以太坊风格 PoS head 共识”的节点模块。
- Proposed Solution: 将 crate 包名从 `node` 迁移为 `agent_world_node`，作为 `agent_world` 的基础模块直接调用。
- Success Criteria:
  - SC-1: 保证在启动模拟（`world_viewer_live`）时自动启动节点，并持续推进 PoS proposal/attestation/commit。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：Node PoS 主循环接入 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 在 `crates/agent_world_node` 内实现 PoS 驱动节点主循环：
  - AC-2: 每 tick 推进 slot。
  - AC-3: 按 slot 选择 proposer。
  - AC-4: 生成 world head proposal。
  - AC-5: 自动补齐 attestation（可配置），在本地闭环提交 head。
  - AC-6: 节点运行时快照增强：输出共识模式、最新高度、提交高度、最后状态、最后错误。
- Non-Goals:
  - 多进程真实网络 gossip。
  - 真实 BLS 聚合签名与经济惩罚执行。
  - fork choice / finality 完整信标链流程。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-node-pos-mainloop.prd.md`
  - `doc/p2p/archive/distributed-node-pos-mainloop.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### NodeConfig（升级后）
- 基础字段：`node_id`、`world_id`、`tick_interval`、`role`。
- PoS 字段：`pos_config`、`auto_attest_all_validators`。

### NodeSnapshot（升级后）
- 运行状态：`running`、`tick_count`、`last_tick_unix_ms`。
- 共识状态：`mode`、`latest_height`、`committed_height`、`slot`、`epoch`、`last_status`、`last_block_hash`。
- 诊断字段：`last_error`。

### NodeRuntime（升级后）
- `start()`：启动线程并推进 PoS。
- `stop()`：停止线程并回收资源。
- `snapshot()`：读取节点与 PoS 的即时状态。

## 5. Risks & Roadmap
- Phased Rollout:
  - NPOS-1：设计文档与项目管理文档落地。
  - NPOS-2：重构 `crates/agent_world_node`，落地 PoS 驱动主循环并更名为 `agent_world_node` 包。
  - NPOS-3：`world_viewer_live` 启动链路接线与测试更新。
  - NPOS-4：回归测试、文档状态收口与 devlog 收口。
- Technical Risks:
  - 单进程模拟“自动补齐 attestation”不等同真实分布式网络，需要在下一阶段补网络传播语义。
  - 若 validator 集配置不当（如 stake 分布导致 proposer 期望与策略冲突），可能出现持续 pending。
  - 包名迁移会影响下游引用与脚本命令，需要同步修正测试命令与文档。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-042-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-042-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
