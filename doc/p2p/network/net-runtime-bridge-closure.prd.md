# Agent World Runtime：`agent_world_net` runtime_bridge 可编译闭环

审计轮次: 5
## 1. Executive Summary
- Problem Statement: 让 `agent_world_net --features runtime_bridge` 在当前拆分架构下可独立编译通过。
- Proposed Solution: 清理 `runtime_bridge` 路径对已删除 runtime 内部模块路径的依赖，改为稳定 crate 依赖。
- Success Criteria:
  - SC-1: 保持既有对外 API 语义不变（bootstrap / head_follow / observer / replay / validation / execution_storage）。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：`agent_world_net` runtime_bridge 可编译闭环 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: `agent_world_net` runtime_bridge 相关模块导入收敛：
  - AC-2: `bootstrap.rs`
  - AC-3: `execution_storage.rs`
  - AC-4: `head_follow.rs`
  - AC-5: `head_validation.rs`
  - AC-6: `observer.rs`
- Non-Goals:
  - 新增分布式协议语义。
  - 改造签名体系、共识机制或 distfs 能力边界。
  - 新增节点编排可执行程序。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/network/net-runtime-bridge-closure.prd.md`
  - `doc/p2p/network/net-runtime-bridge-closure.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
- 保持 `agent_world_net` 现有公开 API 不变：
  - `distributed_bootstrap`
  - `distributed_head_follow`
  - `distributed_observer_replay`
  - `distributed_storage::store_execution_result`
  - `distributed_validation::{validate_head_update, assemble_snapshot, assemble_journal}`
- 依赖来源调整：
  - BlobStore/分片组装：`agent_world_distfs`
  - World/Snapshot/Journal/事件类型：`agent_world::runtime`
  - 协议类型：`agent_world_proto`

## 5. Risks & Roadmap
- Phased Rollout:
  - RB1：完成文档与任务拆解。
  - RB2：完成 runtime_bridge 导入和 feature 依赖收敛，编译通过。
  - RB3：完成定向回归并更新项目状态与开发日志。
- Technical Risks:
  - `agent_world_net -> agent_world` 依赖可能增加层级耦合，需要后续继续下沉抽象。
  - runtime_bridge 代码路径与默认路径存在长期漂移风险，需要后续纳入常规 CI 覆盖。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-085-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-085-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
