# oasis7 Runtime：生产级收口（Gap 1/2/3/4/5/6/8）设计文档

- 对应设计文档: `doc/p2p/distributed/distributed-production-runtime-gap1234568-closure.design.md`
- 对应项目管理文档: `doc/p2p/distributed/distributed-production-runtime-gap1234568-closure.project.md`

审计轮次: 5
## 1. Executive Summary
- Problem Statement: 收口 Gap 1：将节点状态复制从“本地目录侧车”推进为“网络优先拉取 + 本地持久化兜底”。
- Proposed Solution: 收口 Gap 2：补齐生产可用的区块/Blob 交换协议，支持按高度与按内容哈希拉取。
- Success Criteria:
  - SC-1: 收口 Gap 3：把共识提交与执行结果绑定为同一主路径，禁止提交高度先于执行高度落账。
  - SC-2: 收口 Gap 4：关闭默认全员自动代投票，默认改为“仅本节点投票”，其余票据来自网络。
  - SC-3: 收口 Gap 5：复制写入从 single-writer 升级为 epoch-based writer rotation，支持主写者切换。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：生产级收口（Gap 1/2/3/4/5/6/8）设计文档 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: `crates/agent_world_node`
  - AC-2: `PosNodeEngine` 提交/执行强绑定（先执行后落账）。
  - AC-3: 自动投票策略重构：默认仅本节点投票。
  - AC-4: replication record/guard 升级到 writer epoch 语义。
  - AC-5: 新增 replication block exchange 协议（按高度拉 commit、按哈希拉 blob）。
  - AC-6: 新增网络优先补洞同步（gap sync）主路径。
- Non-Goals:
  - 浏览器 wasm32 完整 libp2p 节点实现（本轮不做 Gap 7）。
  - 共识算法本体重写（PoS 阈值模型保持）。
  - 跨机自动发现/自动运维编排（保留静态配置 + 显式参数）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/distributed/distributed-production-runtime-gap1234568-closure.prd.md`
  - `doc/p2p/distributed/distributed-production-runtime-gap1234568-closure.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 1) Replication Writer Epoch
- `FileReplicationRecord` 新增：
  - `writer_epoch: u64`（`>=1`）
- `SingleWriterReplicationGuard` 新增：
  - `writer_epoch: u64`
- 规则：
  - 同 writer 同 epoch：`sequence` 严格递增。
  - writer 变更：`writer_epoch` 必须提升，且新 writer 首条 `sequence=1`。
  - 同 writer epoch 提升：允许重置序列（首条 `sequence=1`）。

### 2) Replication Block Exchange Protocols
- `"/aw/node/replication/fetch-commit/1.0.0"`
  - 请求：`{ world_id, height }`
  - 响应：`{ found, message }`，`message=GossipReplicationMessage`
- `"/aw/node/replication/fetch-blob/1.0.0"`
  - 请求：`{ content_hash }`
  - 响应：`{ found, blob }`

### 3) Commit/Execution Hard Binding
- 节点提交路径调整为：
  - `decision -> execution_hook(on_commit) -> apply_decision`
- 若需要执行绑定且执行失败：
  - 当前提案不落账。
  - 不推进 `committed_height`。

### 4) Storage Challenge Consensus Gate
- 在本地提交复制前执行：
  - 本地 `probe_storage_challenges`。
  - 网络 blob challenge（若启用网络）。
- 门控失败：
  - 拒绝本次提交复制并返回共识错误。

### 5) 运行默认
- `world_viewer_live` 默认：
  - `node_auto_attest_all_validators=false`
  - 默认构建启用分布式网络 feature（见里程碑）。

## 5. Risks & Roadmap
- Phased Rollout:
  - M1：T0 文档冻结（本文件 + 项管文档）。
  - M2：T1 共识执行强绑定 + 默认投票策略收口。
  - M3：T2 writer epoch failover 语义落地。
  - M4：T3/T4 block exchange + 网络补洞主路径。
  - M5：T5 存储挑战共识门控。
  - M6：T6 默认网络集成收口。
  - M7：T7 回归与文档/devlog 收口。
- Technical Risks:
  - 行为变更风险：默认关闭全员自动代投票后，错误拓扑可能出现 pending 提案堆积。
  - 兼容风险：writer epoch 引入后需保持旧快照/旧消息的 serde 兼容。
  - 可用性风险：网络补洞与挑战门控增加网络依赖，需要可观测错误和安全回退。
  - 稳定性风险：默认网络 feature 增加编译与运行负担，需要 required-tier 覆盖。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-084-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-084-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
