# Agent World Runtime：节点执行校验与奖励 Leader/Failover 生产化收口（设计文档）

## 目标
- 收口执行一致性缺口：将节点对等 commit 的执行绑定从“仅透传/记录”提升为“可校验、可拒绝、可在补洞路径重放验证”。
- 收口执行一致性缺口：将节点对等 commit 的执行绑定从“仅透传/记录”提升为“可校验、可拒绝、可在补洞路径执行一致性验证”。
- 收口奖励结算编排缺口：为 reward runtime 引入显式 leader/failover 策略，避免“隐式只有 sequencer 发布”导致的运行不可观测和不可配置。
- 在不依赖完整玩法模块完工的前提下，先把 P2P 基础设施升级为生产语义。

## 范围

### In Scope
- `crates/agent_world_node`
  - 增加执行校验策略配置：
    - `require_execution_on_commit`
    - `require_peer_execution_hashes`
  - 对等 commit 入站校验增强：
    - 可配置要求 commit 必须携带 `execution_block_hash + execution_state_root`。
    - 对已知高度执行绑定进行一致性比对，不一致时拒绝该 commit。
  - replication gap-sync 主路径增强：
    - 补洞时解析完整 commit payload（含 actions）。
    - 校验 `action_root` 与 actions 一致。
    - 校验 payload 的 execution hash/state root 与本地已知执行绑定策略一致（含必填与一致性约束）。
  - `NodeConsensusSnapshot` 增强可观测字段：
    - `last_committed_at_ms`
    - `peer_heads`（包含 node_id/height/committed_at_ms/execution hashes）

- `crates/agent_world`
  - `world_viewer_live` 运行时默认对齐生产执行校验语义：
    - triad / triad_distributed 节点均接 execution hook。
    - 节点配置开启执行绑定与对等执行 hash 要求。
  - reward runtime 增加 leader/failover 策略：
    - 可配置 `leader_node_id`。
    - 可配置 `leader_stale_ms` 与 failover 开关。
    - 当 leader 超时且本节点成为确定性 failover 候选时，允许接管发布 settlement envelope。
    - 报表输出 leader/failover 诊断字段。

### Out of Scope
- 共识算法重写（PoS 出块规则保持不变）。
- 浏览器 wasm32 端完整分布式节点协议栈。
- 跨世界结算与跨分片事务。

## 接口 / 数据

### 1) NodeConfig 新增策略字段
```rust
NodeConfig {
  require_execution_on_commit: bool,
  require_peer_execution_hashes: bool,
}
```

### 2) 节点快照新增字段
```rust
NodeConsensusSnapshot {
  last_committed_at_ms: Option<i64>,
  peer_heads: Vec<NodePeerCommittedHead>,
}

NodePeerCommittedHead {
  node_id: String,
  height: u64,
  block_hash: String,
  committed_at_ms: i64,
  execution_block_hash: Option<String>,
  execution_state_root: Option<String>,
}
```

### 3) reward runtime leader/failover 配置
```rust
CliOptions {
  reward_runtime_leader_node_id: Option<String>,
  reward_runtime_leader_stale_ms: u64,
  reward_runtime_failover_enabled: bool,
}
```

### 4) reward settlement 触发规则（网络模式）
- 仅当满足以下条件才允许本地发布 settlement：
  - 观察者阈值满足。
  - 本地共识状态处于 committed。
  - 本节点是 leader；或 leader 已 stale 且本节点成为 failover 候选发布者。

## 里程碑
- M0：设计与项目管理文档冻结。
- M1：`agent_world_node` 执行校验策略与补洞执行一致性校验落地。
- M2：`world_viewer_live` leader/failover 策略与运行默认语义收口。
- M3：测试回归（required-tier 定向）与文档/devlog 收口。

## 风险
- 严格执行校验可能暴露历史“宽松路径”下未显化的问题，初期可能增加拒绝日志与排障成本。
- failover 判定依赖本地观测视图，极端网络分区下可能出现临时双发布，需要幂等去重兜底。
- triad 全角色执行会增加 CPU/IO 开销，需要后续压测评估阈值。
