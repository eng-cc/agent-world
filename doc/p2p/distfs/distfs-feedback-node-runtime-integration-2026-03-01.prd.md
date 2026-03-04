# Agent World Runtime：DistFS Feedback P2P Node Runtime 接入（2026-03-01）设计文档

## 1. Executive Summary
- Problem Statement: 在不走共识、无中心化服务器前提下，将 feedback 的“提交 + 传播 + 入库”接入 `agent_world_node` 运行时主循环。
- Proposed Solution: 让节点具备两种自动行为：
- Success Criteria:
  - SC-1: 本地提交 feedback 后自动广播 announce。
  - SC-2: 收到远端 announce 后自动按 hash 拉取 blob 并入本地 feedback store。
  - SC-3: 保持 feedback 现有语义：公开读写、append-only、tombstone 逻辑删除、签名作者控制、审计与限流。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：DistFS Feedback P2P Node Runtime 接入（2026-03-01）设计文档 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: `crates/agent_world_node` 新增 feedback p2p 配置：
  - AC-2: feedback store 限流/大小限制配置（复用 `FeedbackStoreConfig`）。
  - AC-3: 每 tick 的 announce drain/publish 上限。
  - AC-4: `NodeRuntime` 内接入 feedback driver：
  - AC-5: 初始化 `FeedbackStore` + `FeedbackAnnounceBridge`。
  - AC-6: tick 内执行 `drain incoming announces -> fetch blob -> ingest`。
- Non-Goals:
  - feedback 内容审核策略升级。
  - 反馈索引查询 API / HTTP 网关。
  - 将 feedback 事件绑定到共识最终性。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/distfs/distfs-feedback-node-runtime-integration-2026-03-01.prd.md`
  - `doc/p2p/distfs/distfs-feedback-node-runtime-integration-2026-03-01.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### Node 配置（草案）
```rust
NodeFeedbackP2pConfig {
  store: FeedbackStoreConfig,
  max_incoming_announces_per_tick: usize,
  max_outgoing_announces_per_tick: usize,
}
```

- feedback store 实际落盘路径复用 replication 本地 store：`<replication.root_dir>/store`。

### NodeRuntime 新增行为
- 启动阶段（`start`）：
  - 当 `config.feedback_p2p` 启用且 `replication_network` 存在时，创建 `FeedbackStore` 与 `FeedbackAnnounceBridge`。
- 运行阶段（每 tick）：
  - 入站：`bridge.drain()` -> 按上限逐条处理 -> 通过 replication fetch-blob 协议拉取 -> `ingest_feedback_announce_with_fetcher`。
  - 出站：从本地 outbox 取 announce -> `bridge.publish()`。
- 提交阶段（对外接口）：
  - `submit/append/tombstone` 直接写 `FeedbackStore`。
  - 依据 receipt 调 `build_feedback_announce_from_receipt` 生成 announce，入 outbox。

### 错误处理
- feedback p2p 未启用时，feedback 提交接口返回 `NodeError::Replication`（明确“feedback p2p 未配置”）。
- 单条 announce ingest 失败不阻断 tick；错误计入 runtime `last_error`（遵循现有 runtime 风格）。

## 5. Risks & Roadmap
- Phased Rollout:
  - M1：T0 设计与项目管理文档完成。
  - M2：T1 NodeConfig + runtime feedback driver 接入完成。
  - M3：T2 feedback 提交 API + 自动广播闭环完成并通过单测。
  - M4：T3 回归、文档状态回写与 devlog 收口完成。
- Technical Risks:
  - fetch-blob 鉴权依赖 replication 配置；若远端开启严格 allowlist，未配置签名请求可能被拒绝。
  - gossip announce 可被刷屏；依赖每 tick 上限、feedback store 限流与签名校验兜底。
  - 无共识模式下为最终一致，不保证节点间实时一致。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-061-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-061-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
