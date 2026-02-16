> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：分布式 Head 共识快照持久化（设计文档）

## 目标
- 为 `QuorumConsensus` 提供可落盘、可恢复的快照能力，解决进程重启后共识状态丢失的问题。
- 保证恢复后的提案/投票状态与阈值规则一致，避免把不安全记录重新加载进内存。
- 在不引入完整链上治理的前提下，为后续成员治理与租约联动提供持久化基础。

## 范围

### In Scope（本次实现）
- 定义共识快照文件结构（版本号、验证者集合、阈值、记录列表）。
- 提供 `QuorumConsensus` 的快照保存与加载接口。
- 加载时校验快照合法性（版本、验证者身份、投票字段一致性），并重算记录终态。
- 提供单元测试覆盖快照 round-trip 和异常快照拒绝路径。

### Out of Scope（本次不做）
- 分布式复制存储与多副本容灾。
- 动态验证者治理流程（提案/投票变更成员集合）。
- 与 `LeaseManager` 的自动联动切换策略。
- BFT/经济激励机制。

## 接口 / 数据

### 快照文件（JSON）
- 字段：
  - `version`: 快照版本（当前 `1`）
  - `validators`: 验证者集合
  - `quorum_threshold`: 法定票数
  - `records`: `HeadConsensusRecord` 列表
- 文件建议名：`consensus_snapshot.json`

### API
- `QuorumConsensus::save_snapshot_to_path(path)`
  - 将当前内存共识状态原子写入 JSON。
- `QuorumConsensus::load_snapshot_from_path(path)`
  - 从快照重建 `QuorumConsensus`。
- `QuorumConsensus::export_records()` / `import_records(...)`
  - 提供内存记录导入导出能力，便于后续接入外部 store。

### 校验规则（加载阶段）
- 快照版本必须匹配。
- `validators` 与 `quorum_threshold` 必须能通过 `QuorumConsensus::new` 安全校验。
- `record.proposer_id` 和每个 vote 的 `validator_id` 必须在验证者集合内。
- vote map key 必须与 vote 内部 `validator_id` 一致。
- 记录状态以票数重算（`Committed/Rejected/Pending`），不盲信快照中的原始状态字段。

## 里程碑
- **CP1**：定义快照文件结构与版本。
- **CP2**：实现保存/加载 API 与校验逻辑。
- **CP3**：补齐单元测试并通过回归。

## 风险
- JSON 快照为单文件，极端大规模记录可能带来加载时延。
- 当前仅本地持久化，不含跨节点防篡改机制。
- 若验证者集合变化频繁，旧快照兼容策略需在后续成员治理中扩展。
