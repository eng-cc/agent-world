# Agent World Runtime：DistFS 路径索引接入 execution_storage

审计轮次: 4
## 1. Executive Summary
- Problem Statement: 将 `agent_world_distfs::FileStore` 接入 `agent_world_net::execution_storage`，为执行产物提供稳定路径索引。
- Proposed Solution: 保持 CAS（content hash）作为底层真相，路径索引只作为可读、可枚举入口。
- Success Criteria:
  - SC-1: 提供最小闭环：执行结果写入时生成路径索引，后续可按 `world_id + height` 回读区块与最新 head。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：DistFS 路径索引接入 execution_storage 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 在 `agent_world_net` 增加“写执行结果 + 写路径索引”的组合接口。
  - AC-2: 约定执行产物路径布局（`worlds/<world_id>/...`）。
  - AC-3: 提供路径读取接口（按高度读 block、读最新 head）。
  - AC-4: 补齐单元测试（写入、回读、非法 world_id）。
- Non-Goals:
  - 跨节点目录同步与冲突解决。
  - 路径索引的版本迁移策略。
  - 目录分页、GC 与冷热分层策略。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/distfs/distfs-runtime-path-index.prd.md`
  - `doc/p2p/distfs/distfs-runtime-path-index.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 新增接口（草案）
- `store_execution_result_with_path_index(...)`：
  - 先复用现有 `store_execution_result` 写 CAS
  - 再写路径索引文件
- `load_block_by_height_from_path_index(world_id, height, store)`：
  - 从路径索引读取 `WorldBlock`
- `load_latest_head_from_path_index(world_id, store)`：
  - 从路径索引读取 `WorldHeadAnnounce`

### 路径约定（草案）
- `worlds/<world_id>/heads/latest_head.cbor`
- `worlds/<world_id>/blocks/<height_20>/block.cbor`
- `worlds/<world_id>/blocks/<height_20>/snapshot_manifest.cbor`
- `worlds/<world_id>/blocks/<height_20>/journal_segments.cbor`

说明：
- `<height_20>` 使用 20 位零填充十进制，保证字典序与高度序一致。
- `world_id` 作为路径分段使用，需进行分段安全校验（限制字符集）。

## 5. Risks & Roadmap
- Phased Rollout:
  - DPRI-1：设计文档与项目管理文档落地。
  - DPRI-2：execution_storage 路径索引写入/读取实现。
  - DPRI-3：单元测试与 crate 级回归。
  - DPRI-4：状态文档与 devlog 收口。
- Technical Risks:
  - `world_id` 兼容性：历史 world_id 若含非法路径字符，会被拒绝写入路径索引。
  - 双写一致性：CAS 写成功但路径索引写失败时，存在“可验证但不可按路径检索”的窗口。
  - 路径布局固定后，后续若需迁移需引入版本化策略。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-076-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-076-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
