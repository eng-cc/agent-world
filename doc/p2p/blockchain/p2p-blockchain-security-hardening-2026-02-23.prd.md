# P2P/区块链链路安全硬化（2026-02-23，设计文档）

审计轮次: 3

## 1. Executive Summary
- Problem Statement: 修复 replication 写入链路中的“先推进 guard 后写入”状态污染问题，保证失败不产生半更新。
- Proposed Solution: 将 Node 共识与复制链路从“签名可选/绑定可选”收口到“生产默认严格绑定与授权”。
- Success Criteria:
  - SC-1: 为 replication `fetch-commit`/`fetch-blob` 增加请求鉴权，降低匿名拉取与枚举风险。
  - SC-2: 为网络订阅队列补齐容量上限，避免高频 topic spam 导致内存无界增长。
  - SC-3: 收紧 membership DHT 恢复默认策略，避免未签名快照在默认路径被接受。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want P2P/区块链链路安全硬化（2026-02-23，设计文档） 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: `crates/agent_world_distfs/src/replication.rs`
  - AC-2: `apply_replication_record` 改为原子语义（失败不污染 guard）。
  - AC-3: `crates/agent_world_node/src/replication.rs`
  - AC-4: 增加远端 writer allowlist 与 fetch 请求签名鉴权数据模型。
  - AC-5: 增加远端复制消息授权校验（签名有效 + writer 授权）。
  - AC-6: `crates/agent_world_node/src/lib.rs`
- Non-Goals:
  - 不引入新密码学算法（继续使用当前 ed25519 / HMAC 体系）。
  - 不改造 libp2p 底层握手协议与 peer 信任模型。
  - 不重构大型文件拆分（本次聚焦安全行为改造与测试闭环）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/blockchain/p2p-blockchain-security-hardening-2026-02-23.prd.md`
  - `doc/p2p/blockchain/p2p-blockchain-security-hardening-2026-02-23.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 1) Replication Guard 原子化
- `apply_replication_record` 调整为：
  - 在局部副本上执行 `validate_and_advance`；
  - 完成 hash 校验与 `store.write_file` 后再提交 guard。
- 失败路径保证：
  - 任何 `Err` 不改变外部 `guard`。

### 2) Signed Consensus 模式下的 signer 绑定完整性
- 当节点启用共识签名强制（由 replication 签名配置驱动）时：
  - `validator_signer_public_keys` 必须覆盖全部 validator；
  - 本地节点 signer 公钥必须与自身 validator 绑定一致；
  - 不满足时启动失败。

### 3) Replication Writer 授权
- `NodeReplicationConfig` 增加远端 writer allowlist（ed25519 公钥 hex 集）。
- 远端 replication 消息校验增加：
  - 签名校验通过；
  - `record.writer_id == signature.public_key_hex`；
  - `record.writer_id` 命中 allowlist。

### 4) Fetch 请求鉴权
- `FetchCommitRequest` / `FetchBlobRequest` 增加鉴权字段（请求方公钥 + 签名）。
- 客户端发起请求时对请求体签名；服务端根据 allowlist 验证签名后处理。
- 缺失或校验失败返回 `ErrBadRequest`。

### 5) 网络订阅容量上限
- `NetworkSubscription` drain inbox 改为有界缓存（超过上限淘汰最旧消息）。
- 默认上限覆盖高频 topic 场景，避免无界 `Vec<Vec<u8>>` 累积。

### 6) Membership DHT 恢复默认策略
- `restore_membership_from_dht` 默认使用 `require_signature=true`。
- 旧行为可通过显式传入宽松策略 API 保留（受调用方控制）。

## 5. Risks & Roadmap
- Phased Rollout:
  - M0：T0 建档完成。
  - M1：T1 完成 replication guard 原子化 + 测试。
  - M2：T2 完成 signed consensus signer 绑定强制 + viewer 配置接线。
  - M3：T3 完成 replication writer allowlist 授权校验。
  - M4：T4 完成 fetch 请求签名鉴权链路。
  - M5：T5 完成网络订阅有界队列与 membership 默认策略收紧。
  - M6：T6 完成测试回归、文档状态回写与 devlog。
- Technical Risks:
  - 兼容性风险：签名与绑定默认收紧后，未配置 signer 映射或混用旧节点会被拒绝，需要同步升级。
  - 运维风险：triad 场景需要稳定可复现的节点密钥派生策略，避免跨进程配置不一致。
  - 可用性风险：鉴权失败会提升拒绝率，需通过清晰错误信息辅助定位配置问题。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-053-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-053-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
