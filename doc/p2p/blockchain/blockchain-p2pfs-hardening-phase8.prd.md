# Agent World Runtime：区块链 + P2P FS 硬改造（Phase 8）设计文档

审计轮次: 3

## 1. Executive Summary
- Problem Statement: 把 membership 与 sequencer 的 ed25519 signer 公钥白名单校验/规范化逻辑抽到共享模块，消除重复实现。
- Proposed Solution: 统一 `signature` 验签返回的 signer 公钥口径（小写规范化 hex），减少调用侧重复 normalize。
- Success Criteria:
  - SC-1: 在不改协议字段的前提下，提高签名治理代码的一致性与可维护性。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：区块链 + P2P FS 硬改造（Phase 8）设计文档 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: **HP8-1：共享公钥治理工具模块**
  - AC-2: 新增 `ed25519` 公钥治理工具（crate 内部）：
  - AC-3: 单个公钥：trim + 非空 + hex 校验 + 32-byte 长度校验 + 小写规范化。
  - AC-4: 白名单集合：逐项规范化与去重，重复项 fail-fast。
  - AC-5: 统一错误口径，保留字段级别可定位信息（例如 `accepted_*[index]`）。
  - AC-6: **HP8-2：调用侧接线统一**
- Non-Goals:
  - CA/证书链、公钥托管、轮换审批流程。
  - 线协议和结构体字段变更（`signature` 字符串格式保持不变）。
  - 引入新的签名算法或多签方案。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase8.prd.md`
  - `doc/p2p/blockchain/blockchain-p2pfs-hardening-phase8.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 新增共享模块（crate 内部）
```rust
normalize_ed25519_public_key_hex(...)
normalize_ed25519_public_key_allowlist(...)
```

### 行为约束
- 公钥统一规范化为小写 64 hex 字符串（对应 32-byte）。
- allowlist 非空时按规范化集合比较 signer 公钥，大小写无关。

## 5. Risks & Roadmap
- Phased Rollout:
  - **HP8-M0**：设计文档 + 项目管理文档。
  - **HP8-M1**：共享模块 + membership/sequencer/signature 接线。
  - **HP8-M2**：测试回归与文档收口。
- Technical Risks:
  - 共享化后若错误口径变化过大，可能影响已有测试断言与运维检索关键词。
  - 验签返回值规范化可能影响依赖原始大小写字符串的边缘逻辑，需要确认调用侧仅做比较语义。
  - 若共享工具实现过度抽象，会降低可读性；本期保持函数粒度小且用途明确。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-052-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-052-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
