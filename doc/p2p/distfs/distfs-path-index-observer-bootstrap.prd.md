# Agent World Runtime：Observer/Bootstrap 路径索引读取接入

审计轮次: 4
## 1. Executive Summary
- Problem Statement: 将 DistFS 路径索引读取能力接入 `agent_world_net` 的 bootstrap/observer 调用链。
- Proposed Solution: 在“本地已持有执行产物”的场景下，支持不依赖网络拉取完成世界恢复。
- Success Criteria:
  - SC-1: 保持现有网络路径不变，新增显式路径索引入口，避免影响既有行为。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：Observer/Bootstrap 路径索引读取接入 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 新增 bootstrap 入口：
  - AC-2: `bootstrap_world_from_head_with_path_index`
  - AC-3: `bootstrap_world_from_latest_path_index`
  - AC-4: 在 `HeadFollower` 增加路径索引同步入口（基于 head 队列选择后应用）。
  - AC-5: 在 `ObserverClient` 增加路径索引同步/报告/循环跟随入口。
  - AC-6: 单元测试覆盖：路径索引恢复成功、latest head 恢复成功。
- Non-Goals:
  - 自动 fallback 策略（网络失败后自动回退路径索引）。
  - 多副本路径索引冲突仲裁。
  - 路径索引 GC/回收策略。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/distfs/distfs-path-index-observer-bootstrap.prd.md`
  - `doc/p2p/distfs/distfs-path-index-observer-bootstrap.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 新增 API（草案）
- `bootstrap_world_from_head_with_path_index(head, store)`：
  - 从路径索引读取 block
  - 从本地 CAS 读取 manifest/segments
  - 复用 `validate_head_update` 校验并构建 `World`
- `bootstrap_world_from_latest_path_index(world_id, store)`：
  - 读取 `latest_head.cbor`，再走 `bootstrap_world_from_head_with_path_index`

### 调用链接入（草案）
- `HeadFollower`：
  - `apply_head_with_path_index`
  - `sync_from_heads_with_path_index`
- `ObserverClient`：
  - `sync_heads_with_path_index`
  - `sync_heads_with_path_index_report`
  - `sync_heads_with_path_index_result`
  - `follow_heads_with_path_index`

## 5. Risks & Roadmap
- Phased Rollout:
  - POBI-1：设计文档与项目管理文档落地。
  - POBI-2：bootstrap/head_follow/observer 路径索引入口实现。
  - POBI-3：单元测试与 `agent_world_net` 回归。
  - POBI-4：状态文档与 devlog 收口。
- Technical Risks:
  - 路径索引与 CAS 数据不一致时，恢复流程会失败（可接受，需错误可诊断）。
  - 新增 API 数量增加，需保持命名与调用意图清晰，避免与网络路径混淆。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-066-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-066-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
