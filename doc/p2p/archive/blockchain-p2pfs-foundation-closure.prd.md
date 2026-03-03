> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：基础区块链 + P2P FS 三步收敛

## 1. Executive Summary
- Problem Statement: 将当前仓库中的 `consensus/net/distfs/node` 能力从“模块可用”收敛为“可运行、可验证、可演进”的基础链路。
- Proposed Solution: 采用三步推进，优先打通主路径闭环，再补安全，最后补最小跨节点文件一致能力。
- Success Criteria:
  - SC-1: 完成 strict 6 章重写并保持语义保真。
  - SC-2: 任务/依赖/状态与 PRD-ID 可追溯。
  - SC-3: 治理检查通过且引用可达。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：基础区块链 + P2P FS 三步收敛 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **第 1 步：主流程闭环（Sequencer Mainloop）**
  - AC-2: 将 `action -> mempool -> batch -> pos head commit -> dht publish` 串成可复用主循环组件。
  - AC-3: 以 `agent_world_consensus` 为主落地，保持与现有 `ActionMempool`、`PosConsensus`、`LeaseManager`、`DistributedDht` 兼容。
  - AC-4: 提供单元测试覆盖：提交动作、按规则出批、提交 head、无动作 idle。
  - AC-5: **第 2 步：签名/验签闭环（最小可信链路）**
  - AC-6: 为 Action/Head/关键投票事件补齐签名生成与验签入口。
- Non-Goals:
  - 全局 BFT、经济激励、复杂惩罚机制。
  - 完整权限系统（ACL/RBAC）和生产级密钥托管体系。
  - 多分片跨链事务与复杂 CRDT 文件系统。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/blockchain-p2pfs-foundation-closure.prd.md`
  - `doc/p2p/archive/blockchain-p2pfs-foundation-closure.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 第 1 步新增接口（草案）
- `SequencerMainloopConfig`
- `SequencerMainloop`
  - `submit_action(action) -> bool`
  - `tick(dht, now_ms) -> SequencerTickReport`

### 第 2 步新增接口（草案）
- `SignatureVerifier`（统一验签接口）
- `SignedEnvelope`（Action/Head 的签名包装）

### 第 3 步新增接口（草案）
- `replicate_file_update(world_id, path, content_hash, version)`
- `apply_replication_record(record)`

## 5. Risks & Roadmap
- Phased Rollout:
  - **BPFS-1**：主流程闭环组件落地（mempool + pos + dht）。
  - **BPFS-2**：签名/验签闭环接线并覆盖拒绝路径测试。
  - **BPFS-3**：DistFS 最小跨节点复制一致能力落地并回归。
- Technical Risks:
  - 第 1 步若与现有 node 主循环语义重叠，需明确边界，避免两套逻辑漂移。
  - 第 2 步若签名策略抽象过重会影响推进节奏，需保持最小可行接口。
  - 第 3 步要避免引入过度复杂的一致性协议，先保证“单写者 + 可恢复”闭环。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-002-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-002-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
