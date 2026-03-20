# oasis7 Runtime：区块链 + P2P FS 硬改造（Phase 2）设计文档

- 对应设计文档: `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase2.design.md`
- 对应项目管理文档: `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase2.project.md`

审计轮次: 5

## 1. Executive Summary
- Problem Statement: 在 Phase 1 网络闭环基础上，补齐“最小可信 + 重启可恢复”能力。
- Proposed Solution: 将节点密钥真正接入 PoS gossip 主链路（proposal/attestation/commit 签名验签）。
- Success Criteria:
  - SC-1: 为 Node PoS 主循环增加状态持久化，支持重启后继续推进高度而非从 1 开始。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：区块链 + P2P FS 硬改造（Phase 2）设计文档 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **HP2-1：Node PoS gossip 签名/验签闭环**
  - AC-2: proposal/attestation/commit 消息增加签名字段（公钥 + 签名）。
  - AC-3: 节点发送侧使用本地签名密钥签名。
  - AC-4: 接收侧在开启签名策略时强制验签，不通过则拒绝消息。
  - AC-5: 保持旧消息兼容（未开启签名策略时可接收无签名消息）。
  - AC-6: **HP2-2：Node PoS 状态持久化**
- Non-Goals:
  - ActionEnvelope/WorldHeadAnnounce 在 `oasis7_consensus` 的签名算法替换（HMAC -> ed25519）与跨 crate 统一签名接口重构。
  - observer 运行态指标到 viewer 运维面板的 UI 展示接线（进入下一阶段）。
  - 多节点身份治理（公钥绑定、信任根、吊销）完整治理闭环。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase2.prd.md`
  - `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase2.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### Gossip 消息签名字段（草案）
- `public_key_hex: Option<String>`
- `signature_hex: Option<String>`

### PoS 持久化文件（草案）
- 路径：`<replication_root>/node_pos_state.json`
- 字段：
  - `next_height`
  - `next_slot`
  - `committed_height`
  - `network_committed_height`
  - `last_broadcast_proposal_height`
  - `last_broadcast_local_attestation_height`
  - `last_broadcast_committed_height`

## 5. Risks & Roadmap
- Phased Rollout:
  - **HP2-0**：设计文档 + 项目管理文档。
  - **HP2-1**：Node PoS gossip 签名/验签闭环完成。
  - **HP2-2**：Node PoS 状态持久化与恢复完成。
  - **HP2-3**：回归测试、文档与 devlog 收口。
- Technical Risks:
  - 签名严格校验开启后，混合版本节点（部分未签名）会被拒绝，需要渐进开关策略。
  - 持久化频率与磁盘写放大会影响高频 tick；本阶段先保证正确性。
  - 状态文件损坏时需容错回退到默认启动路径，避免阻塞节点启动。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-046-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-046-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
