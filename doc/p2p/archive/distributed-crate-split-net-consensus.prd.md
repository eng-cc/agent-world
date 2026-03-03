> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：`agent_world_net` + `agent_world_consensus` 拆分

## 1. Executive Summary
- Problem Statement: 将分布式能力按职责拆为两个 crate：
- Proposed Solution: `agent_world_net`：网络与传输相关能力。
- Success Criteria:
  - SC-1: `agent_world_consensus`：共识与成员同步相关能力。
  - SC-2: 明确将 `distributed_membership_sync` 归类到 `agent_world_consensus`。
  - SC-3: 保持 `agent_world` 对外 API 兼容，避免一次性大范围破坏。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：`agent_world_net` + `agent_world_consensus` 拆分 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 新建 crate：
  - AC-2: `crates/agent_world_net`
  - AC-3: `crates/agent_world_consensus`
  - AC-4: 将两个 crate 加入 workspace，并形成稳定导出入口（按能力分类导出）。
  - AC-5: `agent_world_consensus` 对外聚合共识与 membership sync 能力（含 `distributed_membership_sync` 相关导出）。
  - AC-6: 补充最小验证（编译/测试）与文档回写。
- Non-Goals:
  - 不在本轮强制把 `agent_world` 现有 runtime 实现文件全部物理迁移到新 crate。
  - 不做协议层额外重构（协议仍以 `agent_world_proto` 为主）。
  - 不改动业务语义与现有行为策略。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/archive/distributed-crate-split-net-consensus.prd.md`
  - `doc/p2p/archive/distributed-crate-split-net-consensus.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### `agent_world_net`（边界）
- 负责导出：
  - 网络抽象与 in-memory 网络实现。
  - 分布式网络客户端/网关/观察者等网络链路能力。
  - 可选 `libp2p` 能力导出（feature 透传）。

### `agent_world_consensus`（边界）
- 负责导出：
  - 共识主流程（提案/投票/提交）相关能力。
  - 成员目录同步与恢复能力。
  - 吊销对账、调度、告警、恢复、审计相关能力。
- `distributed_membership_sync` 及其子模块归属此 crate 的能力面。

## 5. Risks & Roadmap
- Phased Rollout:
  - P1：设计文档与项目管理文档完成。
  - P2：新 crate 脚手架与 workspace 接线完成。
  - P3：导出能力面落地（net/consensus 分类稳定）。
  - P4：编译与定向测试回归通过，项目文档收口。
  - P5：`distributed_net` 核心实现下沉到 `agent_world_net`。
  - P6：扩展阶段回归验证与文档收口。
  - P7：`distributed_dht` 核心实现下沉到 `agent_world_net`。
  - P8：二次扩展阶段回归验证与文档收口。
  - P9：`distributed_client` 核心实现下沉到 `agent_world_net`。
  - P10：三次扩展阶段回归验证与文档收口。
- Technical Risks:
  - 仅做边界导出时，可能出现“新 crate 已存在但实现仍在 `agent_world`”的过渡期认知偏差。
  - 导出项过多时，维护成本上升；需要后续按使用频率做分组收敛。
  - feature 透传（尤其 `libp2p`）若配置不完整，可能导致构建行为与预期不一致。
  - 迁移期间若同名类型在 runtime 与新 crate 并行存在，可能引入调用方误用；需要通过文档和导出边界降低歧义。
  - `distributed_client` 涉及协议编解码与错误映射，若 canonical CBOR 行为偏差可能导致跨节点请求兼容性回归，需要定向测试覆盖。
  - `distributed_gateway` 涉及 action 发布 topic 与序列化，若 topic 生成或 payload 编码偏差，可能导致上游节点收不到动作，需要保留端到端发布测试。
  - `distributed_index` 涉及执行产物 hash 聚合与 provider 发布，若 hash 收集集合不一致会导致拉取路径缺 provider，需要保留执行产物全量索引测试。
  - `agent_world_net` 模块化拆分若处理不当可能引入循环依赖或可见性收窄，需保持导出面稳定并保留全量 `agent_world_net` 单测回归。
  - cache/index store 下沉涉及 TTL、provider 截断与 head 缓存刷新路径，若时间窗口判定偏差会导致命中过期数据或频繁回源，需保留缓存命中/过期刷新测试。
  - 观察者链路下沉涉及 head 选择与同步回放入口，若 `world_id` 校验或冲突判定偏差会导致错误 world 被应用，需要保留订阅/同步与 head 冲突路径测试。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-039-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-039-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
