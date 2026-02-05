# Agent World Runtime：分布式计算与存储（设计文档）

## 目标
- 将**计算与存储拆分**为不同节点角色，计算侧专注确定性执行，存储侧专注内容寻址与持久化。
- 参照 **IPFS** 的内容寻址与 DHT 发现方式，以及 **EVM** 的确定性执行与状态根校验模式。
- 保证 WASM 代码、Agent 状态、世界事件、被创造事物的数据**可验证、可复现、可回放**。
- 以 **libp2p** 作为网络层，支持去中心化发现、传输与订阅。

## 范围

### In Scope（V1）
- 节点角色拆分与职责边界（计算/存储/索引/网关）。
- 数据分类与存放策略（WASM 工件、Agent 状态、事件日志、快照、对象数据）。
- 基于内容哈希的存储与引用方式（CAS + DAG 结构）。
- 基于状态根的执行结果可验证机制。
- libp2p 协议层组合（gossipsub + Kademlia + request/response + block exchange）。

### Out of Scope（V1 不做）
- 全局 BFT 共识或经济激励机制。
- 跨世界强一致与复杂跨分片事务。
- 全量加密存储与可验证计算证明（ZK/Fraud Proof）。

## 总体架构

### 节点角色
- **Compute/Execution Node（执行节点）**
  - 运行 world kernel + WASM 模块。
  - 从存储节点拉取 WASM 工件与状态快照。
  - 生成事件日志、收据、状态根与区块元数据。
- **Storage Node（内容存储节点）**
  - 提供内容寻址存储（WASM/快照/日志/对象）。
  - 支持 chunk 化与可校验的传输（hash 校验）。
  - 负责数据持久化、缓存与 pin 策略。
- **Index/Discovery Node（索引发现节点）**
  - 维护世界头（head）索引与内容提供者（provider）索引。
  - 基于 DHT/Kademlia 发布与查询。
- **Sequencer/Coordinator（排序/协调节点）**
  - 为单个世界分片提供动作排序与租约式“单写者”能力。
  - 负责动作批次（ActionBatch）分配到执行节点。
- **Gateway/Observer（网关/观察节点）**
  - 提供提交动作、订阅事件与数据查询入口。

### 运行流程（简述）
1. 网关节点提交 Action → gossipsub 广播到 mempool 或直接给 Sequencer。
2. Sequencer 选择动作批次并分配给 Execution Node。
3. Execution Node 拉取所需 WASM 与状态快照，执行 step，生成事件日志与新状态。
4. Execution Node 写入日志段、快照段与区块元数据到 Storage Node（内容寻址）。
5. Index 节点更新世界 head 指针，观察节点可回放验证状态根。

## 数据分类与存放策略

### 1. WASM 代码与模块元信息
- **存放位置**：Storage Node 的内容寻址存储（CAS）。
- **标识方式**：`wasm_hash = blake3(bytes)`，模块元信息以 `manifest_hash` 关联。
- **访问方式**：DHT 提供者查找 → block exchange 拉取。

### 2. Agent 状态（含模块 state）
- **存放位置**：快照分片（Snapshot Chunks）+ 状态根引用。
- **结构**：`StateChunk { world_id, epoch, chunk_id, data_hash }`。
- **校验方式**：`state_root`（Merkle/DAG root）包含所有 Agent/Module 状态的 hash。

### 3. 世界事件/日志/收据
- **存放位置**：日志分片（Journal Segments）+ 收据分片。
- **结构**：`JournalSegment { from_event_id, to_event_id, segment_hash }`。
- **用途**：回放与审计，验证执行节点输出的事件序列。

### 4. 被创造事物的数据（对象/媒体/模型）
- **小对象**：直接落盘为事件 payload（可压缩）。
- **大对象**：存入 CAS，事件仅记录 `object_hash` 与元信息。
- **引用规则**：事件或对象元数据必须携带 `content_hash`，不可重写。

### 5. 世界头指针与索引
- **世界头**：`WorldHead { world_id, height, block_hash, state_root }`。
- **索引存放**：DHT + 可选的轻量 index store（按世界/分片）。

## 计算与存储的分离原则
- 执行节点**不要求持久化**完整历史，只需缓存热数据与最近快照。
- 存储节点**不执行计算**，只负责内容提供与校验。
- 通过状态根与日志 hash 校验执行结果，确保可重放与可验证。

## 网络层（libp2p）
- **gossipsub**：动作广播、区块/事件头广播。
- **Kademlia DHT**：内容提供者索引、世界 head 索引。
- **block exchange（bitswap/graphsync）**：拉取 WASM/快照/日志分片。
- **request/response**：点对点查询（`get_world_head`、`get_block`、`get_snapshot`）。

## 一致性与验证
- **单写者分片**：每个世界分片在同一时间仅允许一个 Sequencer 持有写入租约。
- **可验证执行**：Execution Node 产出 `state_root`，观察节点可回放并校验。
- **异常处理**：若 state_root 不一致，拒绝该区块并回滚到上一个 head。

## 接口 / 数据（草案）

### WorldBlock（区块元数据）
```
WorldBlock {
  world_id: String,
  height: u64,
  prev_block_hash: String,
  action_root: String,
  event_root: String,
  state_root: String,
  journal_ref: String,
  snapshot_ref: String,
  receipts_root: String,
  proposer_id: String,
  timestamp_ms: i64,
  signature: String
}
```

### SnapshotManifest（快照清单）
```
SnapshotManifest {
  world_id: String,
  epoch: u64,
  chunks: Vec<StateChunkRef>,
  state_root: String
}
```

### 关键 RPC（示意）
- `submit_action(world_id, action)`
- `subscribe_events(world_id, from_height)`
- `get_world_head(world_id)`
- `get_block(world_id, height)`
- `get_snapshot(world_id, epoch)`
- `fetch_blob(content_hash)`

## 里程碑
- **D1**：数据分类与存放策略确认（WASM/状态/日志/对象）。
- **D2**：libp2p 协议组合与 topic/key 约定。
- **D3**：WorldBlock/SnapshotManifest 数据结构冻结。
- **D4**：分片执行与单写者租约设计。
- **D5**：回放验证流程与异常处理策略。

## 风险
- **吞吐瓶颈**：单写者分片会限制并发，需要后续分片扩展。
- **存储膨胀**：日志与对象增长快，需要 GC/pinning 策略。
- **网络复杂度**：libp2p 叠加多协议后调试成本升高。
- **一致性压力**：缺乏 BFT 共识时，节点间可能出现短暂分叉。
