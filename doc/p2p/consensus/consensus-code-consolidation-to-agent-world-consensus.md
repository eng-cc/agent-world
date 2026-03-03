# Agent World Runtime：共识代码统一收敛到 agent_world_consensus（设计文档）

## 目标
- 将 `agent_world_node` 中 PoS 共识核心状态机（proposal/attestation/decision）迁移到 `agent_world_consensus`，避免同语义双实现长期漂移。
- 保持 `agent_world_node` 专注于节点运行时职责（网络收发、复制、执行 hook、快照桥接），共识规则核心改为复用 `agent_world_consensus`。
- 在不破坏现有 runtime 行为的前提下，分阶段完成“代码位置统一 + 运行语义不回退”。

## 范围

### In Scope
- 在 `crates/agent_world_consensus` 新增 node 可复用的 PoS 核心模块：
  - attestation 结构
  - pending proposal 结构
  - decision 结构
  - proposal 构造、attestation 插入、状态推进函数
- `crates/agent_world_node` 改为调用上述核心模块，不再本地维护同构结构与核心推进算法。
- 保持 `agent_world_node` 现有外部 API 与快照结构不变（`NodeSnapshot` / `NodeConsensusSnapshot`）。
- 补齐回归测试，确保现有 PoS 路径与新抽取模块语义一致。

### Out of Scope
- 一次性迁移 `agent_world_node` 全部共识相关网络处理代码（gossip/libp2p endpoint 适配仍留在 node）。
- 改写 `agent_world_consensus::PosConsensus` 到 `agent_world_node` 运行主路径（本轮先做核心抽取，不做大规模替换）。
- Fork-choice/finality/BLS 等完整以太坊信标链语义升级。

## 接口 / 数据

### 1) agent_world_consensus 新增 node_pos 核心模块
- `NodePosAttestation`
- `NodePosPendingProposal<TAction, TStatus>`
- `NodePosDecision<TAction, TStatus>`
- `NodePosStatusAdapter`（用于 node 自定义状态枚举映射）
- 核心函数：
  - `propose_next_head(...)`
  - `advance_pending_attestations(...)`
  - `insert_attestation(...)`
  - `decision_from_proposal(...)`

### 2) agent_world_node 适配
- `PosNodeEngine` 保留运行时状态与网络接线。
- 本地同构结构替换为 `agent_world_consensus::node_pos` 类型别名。
- 错误映射统一为 `NodeError::Consensus { reason }`。

### 3) 依赖分层修正（避免循环依赖）
- 为支持 `agent_world_node -> agent_world_consensus` 单向依赖，`agent_world_consensus` 内聚 `distributed_dht` / `distributed_net` 抽象与内存实现，不再反向依赖 `agent_world_net`。
- 该调整不改变 PoS/成员治理语义，仅收敛 crate 边界，确保后续共识代码可持续集中在 `agent_world_consensus`。

### 4) 第二阶段（尽量一步到位）收口目标
- 将 `agent_world_node` 中残留的共识纯逻辑（action root 计算/校验、共识消息签名验签、共识消息结构定义）迁移到 `agent_world_consensus`，`agent_world_node` 仅保留运行时接线与错误映射。
- 对 `agent_world_consensus` 内部 PoS 双实现关系进行收敛：`node_pos` 作为 node 主链路推进核心，`pos` 复用同一推进核心，避免两套独立推进逻辑长期漂移。
- 保持 `agent_world_node` 外部接口和现有闭环测试口径不回退。

## 里程碑
- CCG-0：设计与项目文档建档。
- CCG-1：抽取 PoS 核心状态机到 `agent_world_consensus::node_pos` 并接线 `agent_world_node`。
- CCG-2：回归测试（node + viewer live 定向）与文档/devlog 收口。
- CCG-3：扩展文档，定义第二阶段“共识代码全收口 + PoS 单链路化”任务。
- CCG-4：迁移 `agent_world_node` 残留共识纯逻辑到 `agent_world_consensus` 并完成接线。
- CCG-5：完成 PoS 内部单链路收敛、定向回归与文档/devlog 终态收口。

## 风险
- 泛型化抽取若边界定义不清，可能导致类型复杂度上升，影响可读性。
- 抽取过程中若状态更新顺序变化，可能引发边界行为回归（如 pending -> committed 时机）。
- 后续若不继续推进网络层抽取，仍会存在“规则已统一、接线分散”的中间状态，需要后续阶段继续收口。
- 第二阶段若迁移边界过大，可能导致 `agent_world_node` 与 `agent_world_consensus` 接口短期震荡，需通过分层回归测试兜底。
