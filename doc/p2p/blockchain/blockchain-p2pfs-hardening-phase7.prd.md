# oasis7 Runtime：区块链 + P2P FS 硬改造（Phase 7）设计文档

- 对应设计文档: `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase7.design.md`
- 对应项目管理文档: `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase7.project.md`

审计轮次: 5
## 1. Executive Summary
- Problem Statement: 将 sequencer `accepted_action_signer_public_keys` 升级为生产级治理语义：配置合法性校验、规范化比较、重复项拒绝。
- Proposed Solution: 统一 ed25519 signer 公钥白名单行为，避免大小写/格式差异导致动作签名误拒绝。
- Success Criteria:
  - SC-1: 保持现有协议兼容，不改变 action/head 线协议字段格式。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：区块链 + P2P FS 硬改造（Phase 7）设计文档 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **HP7-1：Sequencer allowlist 校验与规范化**
  - AC-2: 对 `SequencerMainloopConfig.accepted_action_signer_public_keys` 增加校验：
  - AC-3: 不能为空字符串。
  - AC-4: 必须是合法 32-byte hex ed25519 公钥。
  - AC-5: 按小写规范化后去重，发现重复直接拒绝配置。
  - AC-6: 保留 `require_action_signature` 语义：允许 `hmac_signer` 或 signer 公钥白名单任一满足。
- Non-Goals:
  - 证书链、公钥托管、密钥轮换审批平台。
  - `signature.rs` 对签名串返回值语义改造（本期仅在 sequencer 比较侧规范化）。
  - 新增协议字段或修改 `ActionEnvelope` 数据结构。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase7.prd.md`
  - `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase7.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 配置语义
```rust
SequencerMainloopConfig {
  accepted_action_signer_public_keys: Vec<String>,
  // 其余字段保持不变
}
```

### 新行为
- 配置校验阶段：`accepted_action_signer_public_keys` 必须是 32-byte hex，规范化后不得重复。
- 运行比较阶段：签名中的 signer 公钥规范化后与 allowlist 集合比较（大小写无关）。

## 5. Risks & Roadmap
- Phased Rollout:
  - **HP7-M0**：设计文档 + 项目管理文档。
  - **HP7-M1**：配置校验/规范化实现并接线。
  - **HP7-M2**：测试回归与文档收口。
- Technical Risks:
  - 严格校验会暴露历史脏配置，短期可能引发配置升级失败，需要明确错误口径。
  - 若外部签名实现输出大写 hex，本期需确保比较侧规范化后可兼容。
  - 去重策略改变后，原有包含重复 key 的配置会从“可运行”变为“启动失败”。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-051-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-051-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
