# README P2 缺口收口：Node Replication 统一到 oasis7_net 网络栈

- 对应设计文档: `doc/p2p/node/node-net-stack-unification-readme.design.md`
- 对应项目管理文档: `doc/p2p/node/node-net-stack-unification-readme.project.md`

审计轮次: 5
## 1. Executive Summary
- Problem Statement: 收口此前评估中的“网络栈分裂”问题：将 `oasis7_node` 的 replication libp2p 实现从独立 swarm 线程迁移为复用 `oasis7_net::Libp2pNetwork`。
- Proposed Solution: 保持 `world_viewer_live` 现有接线和配置不变，不改变上层对 `Libp2pReplicationNetwork` 的调用方式。
- Success Criteria:
  - SC-1: 保持 node replication 现有关键语义不回退：
  - SC-2: 多 peer 轮换请求；
  - SC-3: 远端失败后自动重试下一个 peer；

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want README P2 缺口收口：Node Replication 统一到 oasis7_net 网络栈 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: In scope
  - AC-2: `crates/oasis7_node/src/libp2p_replication_network.rs`
  - AC-3: 改为对 `oasis7_net::Libp2pNetwork` 的封装。
  - AC-4: 在封装层补齐 node 语义（轮换/重试/回退）。
  - AC-5: `crates/oasis7_node/Cargo.toml`
  - AC-6: 增加 `oasis7_net`（仅 native 目标依赖）。
- Non-Goals:
  - 不扩展超出原文边界的新需求。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/node/node-net-stack-unification-readme.prd.md`
  - `doc/p2p/node/node-net-stack-unification-readme.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 1) `Libp2pReplicationNetwork` 对外接口保持不变
- 保持已有类型名与 `DistributedNetwork<WorldError>` 实现不变：
  - `Libp2pReplicationNetworkConfig`
  - `Libp2pReplicationNetwork`
- 配置字段保持不变：
  - `keypair`
  - `listen_addrs`
  - `bootstrap_peers`
  - `allow_local_handler_fallback_when_no_peers`

### 2) 请求路由语义（封装层负责）
- 连接 peer 集合非空时：
  - 按轮换顺序选择 peer；
  - 单 peer 请求失败或返回错误响应时，自动重试下一个 peer。
- 连接 peer 集合为空时：
  - 默认返回 `NetworkProtocolUnavailable`（包含 no connected peers 语义）；
  - 仅在 `allow_local_handler_fallback_when_no_peers=true` 时允许本地 handler 回退。

### 3) 远端错误识别
- 底层复用 `oasis7_net::Libp2pNetwork` request/response。
- 封装层在收到响应 payload 后尝试按 `ErrorResponse` 解码：
  - 命中则视为远端处理失败，触发重试或返回 `NetworkRequestFailed`；
  - 未命中则视为业务 payload 成功。

### 4) 依赖图约束
- `oasis7_net` 作为基础网络 crate，不再直接依赖 `oasis7`。
- `runtime_bridge` 在 `oasis7_net` 中保留 feature 名称占位，但不再承载 runtime 绑定实现，避免再次引入包级循环依赖。

## 5. Risks & Roadmap
- Phased Rollout:
  - M1：完成设计文档 + 项目管理文档，冻结迁移边界。
  - M2：完成 node replication 底层迁移与封装层语义补齐。
  - M3：完成 `oasis7_node` / `oasis7_net` 回归与文档/devlog 收口。
- Technical Risks:
  - 兼容风险：迁移后底层 transport 语义由 `oasis7_net` 承担，需通过回归测试确保 node 的重试/回退行为不回退。
  - 误判风险：远端错误以 `ErrorResponse` 识别，需避免将正常 payload 误判为错误响应。
  - 构建风险：`oasis7_node` 新增 `oasis7_net` 依赖时，需限制在 native 目标，避免影响 wasm32 编译路径。
  - 演进风险：runtime bridge 相关能力不再由 `oasis7_net` 直接导出，后续若需要恢复需在独立桥接层实现，避免重新耦合到基础网络层。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-105-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-105-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
