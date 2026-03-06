# Agent World Runtime：DistFS 反馈 P2P 广播与拉取桥接（2026-03-01）设计文档

审计轮次: 5
## 1. Executive Summary
- Problem Statement: 在无中心化服务器场景下，为 feedback 系统补齐“节点间传播”主链路。
- Proposed Solution: 提供 gossip announce 轻量广播，announce 仅携带元信息与 blob 引用。
- Success Criteria:
  - SC-1: 对端基于 announce 中的 content hash 拉取 blob，验证后入本地 feedback store。
  - SC-2: 保持 feedback 原有签名校验、防重放和 append-only/tombstone 语义。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：DistFS 反馈 P2P 广播与拉取桥接（2026-03-01）设计文档 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: `crates/agent_world_distfs` 新增 `feedback_p2p` 模块：
  - AC-2: announce 数据结构与编解码。
  - AC-3: feedback announce topic 约定。
  - AC-4: 从本地 mutation receipt 构造 announce（包含 blob_ref）。
  - AC-5: 对端 ingest：按 hash 拉取 -> hash 校验 -> 解析 root/event -> 回放入库。
  - AC-6: `FeedbackStore` 扩展复制入库接口：
- Non-Goals:
  - DHT provider 策略优化与多源拉取调度。
  - 共识层最终性绑定。
  - 内容审核、风控策略升级。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/distfs/distfs-feedback-p2p-bridge-2026-03-01.prd.md`
  - `doc/p2p/distfs/distfs-feedback-p2p-bridge-2026-03-01.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### Topic 命名
- `aw.<world_id>.feedback.announce`

### Announce 结构（草案）
```rust
FeedbackAnnounce {
  version: u8,
  world_id: String,
  feedback_id: String,
  action: FeedbackActionKind, // create|append|tombstone
  event_id: String,
  actor_public_key_hex: String,
  blob_ref: FeedbackBlobRef,
  emitted_at_ms: i64,
}

FeedbackBlobRef {
  path: String,
  content_hash: String,
  size_bytes: u64,
}
```

### Ingest 规则
- 拉取 blob 后先验证 `blake3(blob_bytes) == blob_ref.content_hash`。
- `create`：blob 解析为 `FeedbackRootRecord`，执行复制入库。
- `append/tombstone`：blob 解析为 `FeedbackEventRecord`，执行复制入库。
- 重复 announce：以 `feedback_id + event_id` 去重，幂等返回。

## 5. Risks & Roadmap
- Phased Rollout:
  - M1：T0 文档与任务拆解完成。
  - M2：T1 feedback store 复制入库能力完成并通过单测。
  - M3：T2 P2P announce/ingest 桥接模块完成并通过单测。
  - M4：T3 回归、文档/devlog 收口完成。
- Technical Risks:
  - 远端 announce 可被垃圾广播；依赖 blob/hash 校验 + store 层签名校验兜底。
  - 复制入库与本地限流策略语义不同；复制路径需跳过 IP/pubkey 限流。
  - 无共识模式下仅最终一致，不保证全节点实时一致。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-063-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-063-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
