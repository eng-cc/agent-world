# Agent World Runtime：DistFS 公开反馈账本（2026-03-01）设计文档

## 1. Executive Summary
- Problem Statement: 在不引入数据库的前提下，基于 `agent_world_distfs` 落地可运行的玩家反馈系统。
- Proposed Solution: 支持公开写入与公开读取，同时保证反馈记录 append-only，不允许覆盖原始反馈。
- Success Criteria:
  - SC-1: 支持作者控制：反馈追加与删除（tombstone）必须由原作者公钥签名授权。
  - SC-2: 支持基础 anti-abuse：IP 与 pubkey 双限流、内容大小限制、附件数量与大小限制。
  - SC-3: 支持审计：所有变更事件写入审计日志，便于后续排障与治理。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：DistFS 公开反馈账本（2026-03-01）设计文档 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: `crates/agent_world_distfs` 新增反馈模块（核心存储与校验逻辑）：
  - AC-2: feedback create / append / tombstone。
  - AC-3: Ed25519 签名校验（pubkey + signature）。
  - AC-4: nonce 重放保护（同 pubkey + nonce 只允许一次）。
  - AC-5: 基于审计日志的 IP/pubkey 时间窗限流。
  - AC-6: 公开读接口（list / get）。
- Non-Goals:
  - 内容审核与 AI 风险过滤。
  - 多节点跨机复制一致性策略优化。
  - 附件二进制上传与媒体转码流程。
  - 权限分级读（本期按需求采用全量公开读）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/distfs/distfs-feedback-open-ledger-2026-03-01.prd.md`
  - `doc/p2p/distfs/distfs-feedback-open-ledger-2026-03-01.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 目录布局（DistFS FileStore）
- `feedback/records/<feedback_id>/root.json`：反馈创建根记录（不可变）。
- `feedback/records/<feedback_id>/events/<event_id>.json`：追加/删除事件（append-only）。
- `feedback/nonces/<pubkey>/<nonce>.json`：nonce 占位，防重放。
- `feedback/audit/<audit_id>.json`：审计日志（append-only）。

### 关键数据结构（草案）
```rust
FeedbackRootRecord {
  feedback_id: String,
  author_public_key_hex: String,
  created_at_ms: i64,
  content: String,
  category: String,
  platform: String,
  game_version: String,
  attachments: Vec<FeedbackAttachment>,
  signature_hex: String,
  nonce: String,
  expires_at_ms: i64,
}

FeedbackEvent {
  feedback_id: String,
  event_id: String,
  action: "append" | "tombstone",
  actor_public_key_hex: String,
  created_at_ms: i64,
  payload: Option<String>,
  reason: Option<String>,
  signature_hex: String,
  nonce: String,
  expires_at_ms: i64,
}
```

### 签名与防重放
- 仅支持 `Ed25519`。
- 签名 payload 使用 canonical CBOR 编码，固定包含：
  - `action`
  - `feedback_id`
  - `content_hash`（或 tombstone reason hash）
  - `nonce`
  - `timestamp_ms`
  - `expires_at_ms`
- 校验规则：
  - `now_ms <= expires_at_ms`。
  - `nonce` 未被该 pubkey 使用过。
  - 签名与 pubkey 可验证。

### 限流策略（基础）
- 时间窗：`rate_limit_window_ms`（默认 60_000）。
- 阈值：
  - `max_actions_per_ip_window`（默认 20）。
  - `max_actions_per_pubkey_window`（默认 10）。
- 统计来源：扫描时间窗内 `feedback/audit/*.json` 的 accepted mutation 事件。

### 删除语义
- 删除仅写 tombstone 事件，不物理删除 root/event 文件。
- 公共读取视图中返回 `tombstoned=true` 与 `tombstone_reason`。

## 5. Risks & Roadmap
- Phased Rollout:
  - M1：T0 文档建档完成（设计 + 项目管理）。
  - M2：T1 核心反馈模块完成并通过 `agent_world_distfs` 定向单测。
  - M3：T2 CLI 与闭环测试完成，文档/devlog 收口。
- Technical Risks:
  - 审计日志扫描限流在高并发场景会有性能开销；本期接受，后续可改为滚动索引。
  - 全量公开读可能出现恶意内容暴露；本期按需求保持公开，后续可扩展审核层。
  - nonce 占位在并发竞争下依赖文件写时序；本期以单进程/低并发为目标。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-062-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-062-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
