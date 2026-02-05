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

## Mempool 聚合与去重（草案）
- **去重键**：以 `action_id` 为唯一键（V1）。
- **来源限制**：同一 `actor_id` 的待处理动作数有上限（防刷/压制）。
- **容量控制**：mempool 满时按到达顺序驱逐最旧动作。
- **批次排序**：按 `timestamp_ms` 升序，`action_id` 作为稳定 tie-breaker。
- **批次 ID**：对动作 ID 列表做 canonical CBOR 后取 `blake3`（V1）。
- **批次上限**：限制 `max_actions` 与 `max_payload_bytes`，超限动作跳过/剔除。

## 租约式单写者切换（草案）
- **租约模型**：Sequencer 持有带 TTL 的 lease，过期后可被其他节点接管。
- **续约机制**：持有者在 TTL 内续约，否则视为失效。
- **接管条件**：当 lease 过期或显式释放时，其他 Sequencer 可尝试获取。
- **冲突处理**：未过期时拒绝并返回当前 lease 信息。
- **幂等要求**：基于 `lease_id` 续约/释放，避免重复操作。

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

## BlobStore 接口（草案）
- **基础能力**：`put(content_hash, bytes)` / `get(content_hash)` / `has(content_hash)`。
- **内容哈希**：V1 统一 `blake3(bytes)`，十六进制字符串作为内容键。
- **校验策略**：`put` 时必须校验 hash；不一致直接拒绝并报错。
- **本地 CAS 布局**：`<root>/blobs/<content_hash>.blob`（原子写入）。
- **错误语义**：`not_found` / `hash_mismatch` / `hash_invalid`。
- **Pin/Evict**：本地 CAS 维护 `pins.json`；`prune_unpinned(max_bytes)` 仅驱逐未 pinned 且最旧的 blob。

## 快照/日志分片策略（草案）
- **快照分片**：对 `Snapshot` 进行 canonical CBOR 编码后按字节切分。
- **日志分片**：按事件数量切分（默认 256 条/段），每段 CBOR 编码后落盘。
- **默认参数**：`snapshot_chunk_bytes=256KiB`，`journal_events_per_segment=256`。
- **索引信息**：快照使用 `SnapshotManifest` 记录 chunk 列表；日志分片记录 `from_event_id/to_event_id`。
- **一致性**：`state_root` 采用快照 CBOR 的 `blake3` hash（V1 过渡方案）。

## 执行结果写入 storage（草案）
- **快照落盘**：执行节点完成 step 后生成 `Snapshot`，按分片策略写入 CAS。
- **快照索引**：`SnapshotManifest` CBOR 存入 CAS，`snapshot_ref = hash(manifest)`。
- **日志落盘**：`Journal` 按事件分段写入 CAS，形成 `JournalSegmentRef` 列表。
- **日志索引**：分段列表 CBOR 存入 CAS，`journal_ref = hash(segments)`。
- **区块元数据**：生成 `WorldBlock`（包含 action/event/receipts/state roots 与 refs），CBOR 存入 CAS。
- **Root 规则（V1）**：
  - `action_root = hash(ActionId 列表 CBOR)`
  - `event_root = hash(事件列表 CBOR)`
  - `receipts_root = hash(Receipt 列表 CBOR)`
- **广播对象**：由区块派生 `BlockAnnounce` 与 `WorldHeadAnnounce` 用于 gossipsub。

## 网络层（libp2p）
- **gossipsub**：动作广播、区块/事件头广播。
- **Kademlia DHT**：内容提供者索引、世界 head 索引。
- **block exchange（bitswap/graphsync）**：拉取 WASM/快照/日志分片。
- **request/response**：点对点查询（`get_world_head`、`get_block`、`get_snapshot`）。

### 网络适配器（草案）
- **抽象接口**：`DistributedNetwork`（publish/subscribe/request/register_handler）。
- **本地实现**：`InMemoryNetwork`（单进程测试用，支持消息投递与 rr handler）。
- **错误语义**：无 handler 时返回 `NetworkProtocolUnavailable`。
- **客户端封装**：`DistributedClient` 负责封装 rr 请求/响应与 CBOR 编解码。
- **libp2p 骨架**：`Libp2pNetwork`（feature=libp2p），先提供 peer_id/keypair 与接口占位。
  - **v0 事件循环**：后台线程驱动 Swarm，处理 gossipsub 消息与 request/response。
  - **RR 协议**：暂用 `/aw/rr/1.0.0` 单协议承载所有方法，method 放在请求体内（后续可拆分）。

### DHT 适配器（草案）
- **抽象接口**：`DistributedDht`（publish_provider/get_providers/put_world_head/get_world_head）。
- **本地实现**：`InMemoryDht`（用于测试与最小闭环）。
- **索引对象**：`ProviderRecord { provider_id, last_seen_ms }`。

### 协议命名约定（草案）
- **Topic 命名**：`aw.<world_id>.<kind>`（例如 `aw.w1.action`、`aw.w1.block`、`aw.w1.head`）。
- **Request/Response 协议**：`/aw/rr/1.0.0/<method>`。
- **DHT Key**：`/aw/world/<world_id>/<key>`，例如 `head`、`providers/<content_hash>`。
- **内容哈希**：V1 使用 `blake3` 十六进制字符串；后续可升级为 CIDv1（保留兼容层）。

### Gossipsub Topics（草案）
- `aw.<world_id>.action`：ActionEnvelope 广播（mempool 输入）。
- `aw.<world_id>.block`：WorldBlock/BlockAnnounce 广播（高度与 hash）。
- `aw.<world_id>.head`：WorldHeadAnnounce 广播（头指针更新）。
- `aw.<world_id>.event`：EventAnnounce 广播（轻量事件摘要）。

### Request/Response 协议（草案）
- `/aw/rr/1.0.0/get_world_head`
- `/aw/rr/1.0.0/get_block`
- `/aw/rr/1.0.0/get_snapshot`
- `/aw/rr/1.0.0/get_journal_segment`
- `/aw/rr/1.0.0/get_receipt_segment`
- `/aw/rr/1.0.0/fetch_blob`
- `/aw/rr/1.0.0/get_module_manifest`
- `/aw/rr/1.0.0/get_module_artifact`

## 一致性与验证
- **单写者分片**：每个世界分片在同一时间仅允许一个 Sequencer 持有写入租约。
- **可验证执行**：Execution Node 产出 `state_root`，观察节点可回放并校验。
- **Head 更新校验**：收到新 head 后拉取 block/manifest/journal，重算 roots 与 block_hash，通过后才更新 head/DHT。
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

### ActionEnvelope（动作封装）
```
ActionEnvelope {
  world_id: String,
  action_id: String,
  actor_id: String,
  action_kind: String,
  payload_cbor: Bytes,
  payload_hash: String,
  nonce: u64,
  timestamp_ms: i64,
  signature: String
}
```

### ActionBatch（排序批次）
```
ActionBatch {
  world_id: String,
  batch_id: String,
  actions: Vec<ActionEnvelope>,
  proposer_id: String,
  timestamp_ms: i64,
  signature: String
}
```

### WorldHeadAnnounce（头指针广播）
```
WorldHeadAnnounce {
  world_id: String,
  height: u64,
  block_hash: String,
  state_root: String,
  timestamp_ms: i64,
  signature: String
}
```

### BlockAnnounce（区块广播）
```
BlockAnnounce {
  world_id: String,
  height: u64,
  block_hash: String,
  prev_block_hash: String,
  state_root: String,
  event_root: String,
  timestamp_ms: i64,
  signature: String
}
```

### BlobRef（内容寻址引用）
```
BlobRef {
  content_hash: String,
  size_bytes: u64,
  codec: String,
  links: Vec<String>
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

### Request/Response（示意）
```
GetWorldHeadRequest { world_id: String }
GetWorldHeadResponse { head: WorldHeadAnnounce }

GetBlockRequest { world_id: String, height: u64 }
GetBlockResponse { block: WorldBlock, journal_ref: String, snapshot_ref: String }

FetchBlobRequest { content_hash: String }
FetchBlobResponse { blob: Bytes, content_hash: String }

ErrorResponse { code: String, message: String, retryable: bool }
```

### 关键 RPC（示意）
- `submit_action(world_id, action)`
- `subscribe_events(world_id, from_height)`
- `get_world_head(world_id)`
- `get_block(world_id, height)`
- `get_snapshot(world_id, epoch)`
- `fetch_blob(content_hash)`

## 错误码与重试语义（草案）
- `ERR_NOT_FOUND`：找不到资源或区块。
- `ERR_BAD_REQUEST`：请求参数非法或字段缺失。
- `ERR_INVALID_HASH`：content_hash 校验失败。
- `ERR_STATE_MISMATCH`：state_root 不一致或高度冲突。
- `ERR_UNSUPPORTED`：协议版本或方法不支持。
- `ERR_UNAUTHORIZED`：签名或权限校验失败。
- `ERR_BUSY`：节点繁忙，可重试。
- `ERR_RATE_LIMITED`：限流拒绝，可退避重试。
- `ERR_TIMEOUT`：超时失败，可重试。
- `ERR_NOT_AVAILABLE`：服务暂不可用，可重试。

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
