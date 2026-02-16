> [!WARNING]
> 该文档已过期，仅供历史追溯，不再作为当前实现依据。
> 归档日期：2026-02-16

# Agent World Runtime：基础区块链 + P2P FS 三步收敛（设计文档）

## 目标
- 将当前仓库中的 `consensus/net/distfs/node` 能力从“模块可用”收敛为“可运行、可验证、可演进”的基础链路。
- 采用三步推进，优先打通主路径闭环，再补安全，最后补最小跨节点文件一致能力。

## 范围

### In Scope
- **第 1 步：主流程闭环（Sequencer Mainloop）**
  - 将 `action -> mempool -> batch -> pos head commit -> dht publish` 串成可复用主循环组件。
  - 以 `agent_world_consensus` 为主落地，保持与现有 `ActionMempool`、`PosConsensus`、`LeaseManager`、`DistributedDht` 兼容。
  - 提供单元测试覆盖：提交动作、按规则出批、提交 head、无动作 idle。

- **第 2 步：签名/验签闭环（最小可信链路）**
  - 为 Action/Head/关键投票事件补齐签名生成与验签入口。
  - 引入统一签名策略接口，默认提供本地可测实现（先最小化，不做复杂密钥基础设施）。
  - 在网关和主循环中接入“验签失败拒绝”路径。

- **第 3 步：DistFS 最小跨节点复制一致**
  - 在现有 CAS + 路径索引基础上增加最小副本同步协议（单写者语义）。
  - 增加冲突保护（基于版本戳或单调序号）与回放恢复测试。
  - 保持本地路径索引模式可独立工作。

### Out of Scope
- 全局 BFT、经济激励、复杂惩罚机制。
- 完整权限系统（ACL/RBAC）和生产级密钥托管体系。
- 多分片跨链事务与复杂 CRDT 文件系统。

## 接口 / 数据

### 第 1 步新增接口（草案）
- `SequencerMainloopConfig`
- `SequencerMainloop`
  - `submit_action(action) -> bool`
  - `tick(dht, now_ms) -> SequencerTickReport`

### 第 2 步新增接口（草案）
- `SignatureVerifier`（统一验签接口）
- `SignedEnvelope`（Action/Head 的签名包装）

### 第 3 步新增接口（草案）
- `replicate_file_update(world_id, path, content_hash, version)`
- `apply_replication_record(record)`

## 里程碑
- **BPFS-1**：主流程闭环组件落地（mempool + pos + dht）。
- **BPFS-2**：签名/验签闭环接线并覆盖拒绝路径测试。
- **BPFS-3**：DistFS 最小跨节点复制一致能力落地并回归。

## 风险
- 第 1 步若与现有 node 主循环语义重叠，需明确边界，避免两套逻辑漂移。
- 第 2 步若签名策略抽象过重会影响推进节奏，需保持最小可行接口。
- 第 3 步要避免引入过度复杂的一致性协议，先保证“单写者 + 可恢复”闭环。
