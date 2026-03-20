# Node 共识签名身份绑定与复制摄取硬化

- 对应设计文档: `doc/p2p/node/node-consensus-signer-binding-replication-hardening.design.md`
- 对应项目管理文档: `doc/p2p/node/node-consensus-signer-binding-replication-hardening.project.md`

审计轮次: 5
## 1. Executive Summary
- Problem Statement: 收口 P0：把 node 共识消息签名从“仅校验签名有效”升级为“校验签名有效 + 校验签名公钥与 validator 身份绑定一致”，阻断伪造 validator 身份的空间。
- Proposed Solution: 收口 P1-1：把 replication 入站处理从“先更新网络高度再尝试落库、且落库错误被吞”升级为“先成功校验/落库再更新观测高度，错误可观测”。
- Success Criteria:
  - SC-1: 收口 P1-2：把启动阶段 PoS 状态恢复从“加载失败静默忽略”升级为“加载失败立即报错并阻止启动”。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Node 共识签名身份绑定与复制摄取硬化 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: `crates/oasis7_node/src/types.rs`
  - AC-2: 为 `NodePosConfig` 增加 validator->signer 公钥绑定配置。
  - AC-3: `crates/oasis7_node/src/pos_validation.rs`
  - AC-4: 增加 signer 公钥格式与归一化校验。
  - AC-5: 校验 signer 绑定完整性（启用时要求与 validator 集一致）。
  - AC-6: `crates/oasis7_node/src/lib.rs`
- Non-Goals:
  - 不重写共识协议，不引入新密码学算法。
  - 不改动 `oasis7_consensus` 的签名策略配置模型。
  - 不引入外部 KMS 或证书系统。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/node/node-consensus-signer-binding-replication-hardening.prd.md`
  - `doc/p2p/node/node-consensus-signer-binding-replication-hardening.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 1) Validator Signer 绑定
- `NodePosConfig` 新增字段：
  - `validator_signer_public_keys: BTreeMap<String, String>`
- 语义：
  - key: `validator_id`
  - value: ed25519 公钥 hex（32-byte，允许大小写输入，内部归一化为小写）
- 校验规则：
  - 若 map 为空：保持兼容，不强制 signer 绑定。
  - 若 map 非空：必须覆盖全部 validator，且不能包含未知 validator。
  - map 值必须是合法 32-byte hex。

### 2) 共识消息验签增强
- 对 `proposal/attestation/commit`：
  1. 先执行现有签名校验。
  2. 再执行 `validator_id -> signer_public_key` 绑定校验。
- 绑定启用时（`validator_signer_public_keys` 非空）：
  - 消息必须携带 `public_key_hex`，且归一化后与配置绑定值一致。
  - 不一致则丢弃该消息。

### 3) Replication ingest 顺序与错误模型
- `ingest_network_replications` 行为调整：
  - 仅在 `apply_remote_message` 成功后，才更新 `network_committed_height/peer_heads` 与本地同步推进。
  - `apply_remote_message` 失败不再静默，转化为 `NodeError::Replication` 上抛（聚合错误摘要）。

### 4) 启动恢复错误显式化
- `NodeRuntime::start` 中 `PosNodeStateStore::load`：
  - `Err` 直接返回启动失败。
  - 禁止继续以“默认状态”悄然启动。

## 5. Risks & Roadmap
- Phased Rollout:
  - M1：T0 文档建档（本设计 + 项管）。
  - M2：T1 完成 P0 数据模型与校验链路改造。
  - M3：T2 完成 P0 测试闭环。
  - M4：T3 完成 P1 replication ingest 与启动恢复硬化。
  - M5：T4 完成 P1 测试与回归收口。
- Technical Risks:
  - 兼容风险：启用 signer 绑定后，旧节点若未配置公钥映射将无法享受新防护，需要明确默认兼容策略。
  - 运维风险：错误配置 signer map 会导致消息被拒，需要在配置校验阶段尽早失败并给出明确信息。
  - 可用性风险：上抛 replication 错误会提升噪音，需要聚合错误文本避免日志风暴。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-088-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-088-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
