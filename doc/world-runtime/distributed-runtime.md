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

## Gateway API（草案）
- **提交动作**：网关将 `ActionEnvelope` 以 CBOR 编码发布到 `aw.<world_id>.action`。
- **回执**：返回 `SubmitActionReceipt { action_id, accepted_at_ms }`。

## Observer 订阅（草案）
- **订阅事件**：订阅 `aw.<world_id>.event` 获取轻量事件流。
- **订阅头指针**：订阅 `aw.<world_id>.head` 获取世界头更新。

## 观察者回放验证（草案）
- **拉取链路**：`get_world_head` → `get_block` → `fetch_blob(snapshot_ref/journal_ref)`。
- **校验流程**：重算 roots 与 block_hash，复原 snapshot/journal 并回放校验。
- **DHT 引导**：可先从 DHT/index store 读取 head（CachedDht），再用 rr 拉取区块与分片。
- **Provider 选择**：若 DHT 返回 provider 列表，优先向 provider peers 请求 blob。

## 执行节点启动引导（草案）
- **目标**：从 DHT/head 拉取快照与日志，重建可执行世界状态。
- **流程**：`bootstrap_world_from_dht` → 校验 head/块/快照/日志 → `World::from_snapshot`。
- **用途**：执行节点冷启动或重启时快速追上当前 head。

## Head 跟随与同步（草案）
- **目标**：处理 head 广播的乱序/重复，选择最新 head 并同步本地世界。
- **选择规则**：按 `height` 最大优先；同高按 `timestamp_ms` 最大优先；仍冲突时按 `block_hash` 字典序最大优先。
- **冲突处理**：同一 `height` 出现不同 `block_hash` 视为冲突，拒绝并返回错误（需要外部治理或人工介入）。
- **同步策略**：对选中的 head 走 bootstrap 校验流程，重建 `World` 并更新本地 head 视图。
- **Observer 接入**：`ObserverClient::sync_heads`/`sync_heads_with_dht` 使用 `HeadFollower` 从订阅队列中选 head 并触发 bootstrap。
- **结果回传**：`ObserverClient::sync_heads_with_result` 可返回已应用的 head 与重建后的 `World`，便于日志/度量。
- **同步报告**：`ObserverClient::sync_heads_report` 返回已消费 head 数量与是否应用，用于运行状态观测。
- **循环跟随**：`ObserverClient::follow_heads`/`follow_heads_with_dht` 在最多 N 轮 drain 中持续同步，汇总消耗数量与最后应用 head。

## 租约式单写者切换（草案）
- **租约模型**：Sequencer 持有带 TTL 的 lease，过期后可被其他节点接管。
- **续约机制**：持有者在 TTL 内续约，否则视为失效。
- **接管条件**：当 lease 过期或显式释放时，其他 Sequencer 可尝试获取。
- **冲突处理**：未过期时拒绝并返回当前 lease 信息。
- **幂等要求**：基于 `lease_id` 续约/释放，避免重复操作。

## 成员目录广播与同步（草案）
- **广播主题**：`aw.<world_id>.membership`，用于传播最新 validator 目录与 quorum 阈值。
- **广播载荷**：`MembershipDirectoryAnnounce { requester_id, requested_at_ms, validators, quorum_threshold }`。
- **同步策略**：订阅节点将广播目录转换为 `ReplaceValidators` 并尝试应用到本地 `QuorumConsensus`。
- **幂等语义**：若目录未变化则记为 ignored；目录变化成功则记为 applied。
- **安全约束**：若本地存在 pending 提案，沿用共识层保护策略，阻断目录切换。

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
 - **Head 发布**：执行节点完成校验后调用 `put_world_head` 更新 DHT。
 - **Provider 索引**：storage 节点对 block/manifest/segments/chunks 发布 provider 记录。

### DHT 缓存封装（草案）
- **封装目标**：在 DHT 查询前优先读取本地 index store，降低查询成本。
- **缓存内容**：provider 列表与 world head。
- **TTL 策略**：`provider_ttl_ms` 与 `head_ttl_ms` 分别控制有效期；过期后回源 DHT。
- **回写策略**：回源结果写入 index store 以便后续命中。

### 轻量 Index Store（草案）
- **用途**：缓存 world head 与 provider 查询结果（可选）。
- **抽象接口**：`DistributedIndexStore`（put_head/get_head/put_provider/get_providers）。
- **本地实现**：`InMemoryIndexStore` 便于测试。

### Provider 缓存与重发（草案）
- **缓存策略**：查询 providers 时先读本地 cache，命中且未过期直接返回。
- **缓存时间**：以 `last_seen_ms` 与 `provider_ttl_ms` 判断有效性，过期后回源 DHT。
- **重发策略**：本地已缓存或已持有内容的节点可定期 `republish` 自身 provider 记录。
- **触发方式**：定时任务或执行写入后批量触发，避免 DHT 记录过期。
- **执行接入**：执行结果落盘后，通过 `ProviderCache` 批量注册 block/manifest/segments/chunks。
- **拉取封装**：执行/观察者拉取 blob 时，可通过 `DistributedClient::fetch_blob_from_dht` 统一走 provider-aware 路径。
- **模块拉取**：模块 manifest/wasm 工件同样通过 DHT provider 列表拉取，避免固定单点。
- **libp2p 重发**：libp2p 侧可按 `republish_interval_ms` 定期 `start_providing`，维持 provider 记录活性。
- **执行加载**：执行节点加载模块时若缺少工件，可调用 `World::load_module_with_fetch` 触发 DHT 拉取并注册。
- **预热加载**：执行节点启动时可调用 `World::prefetch_active_modules_with_fetch` 预热活跃模块工件。
- **治理闭环**：治理 shadow/apply 可使用 `shadow_proposal_with_fetch` / `apply_proposal_with_fetch` 自动拉取缺失工件。

### 协议命名约定（草案）
- **Topic 命名**：`aw.<world_id>.<kind>`（例如 `aw.w1.action`、`aw.w1.block`、`aw.w1.head`、`aw.w1.membership`、`aw.w1.membership.revoke`、`aw.w1.membership.reconcile`）。
- **Request/Response 协议**：`/aw/rr/1.0.0/<method>`。
- **DHT Key**：`/aw/world/<world_id>/<key>`，例如 `head`、`providers/<content_hash>`。
- **成员目录快照 Key**：`/aw/world/<world_id>/membership`。
- **内容哈希**：V1 使用 `blake3` 十六进制字符串；后续可升级为 CIDv1（保留兼容层）。

### Gossipsub Topics（草案）
- `aw.<world_id>.action`：ActionEnvelope 广播（mempool 输入）。
- `aw.<world_id>.block`：WorldBlock/BlockAnnounce 广播（高度与 hash）。
- `aw.<world_id>.head`：WorldHeadAnnounce 广播（头指针更新）。
- `aw.<world_id>.event`：EventAnnounce 广播（轻量事件摘要）。
- `aw.<world_id>.membership`：成员目录广播（validator 集合与 quorum 阈值）。
- `aw.<world_id>.membership.revoke`：成员目录签名 key 吊销广播（key_id、requester、reason）。
- `aw.<world_id>.membership.reconcile`：成员目录吊销状态对账广播（node_id、revoked_key_ids、hash）。

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

## 成员目录 DHT 快照与恢复（草案）
- **DHT Key**：`/aw/world/<world_id>/membership`，保存最近一次成员目录快照。
- **快照结构**：`MembershipDirectorySnapshot { requester_id, requested_at_ms, validators, quorum_threshold }`。
- **发布联动**：成员变更广播后，调用 `publish_membership_change_with_dht` 同步写入 DHT。
- **恢复入口**：启动/重启时可调用 `restore_membership_from_dht`，读取快照并以 `ReplaceValidators` 恢复本地目录。
- **缺省行为**：DHT 无快照时返回 `None`，不强制变更本地目录。
- **一致性约束**：恢复仍受共识层 pending 保护，避免在进行中提案期间切换 validator 集合。

## 成员目录快照签名与来源校验（草案）
- **签名字段**：成员目录广播与 DHT 快照增加可选 `signature`（hex），兼容旧数据。
- **签名算法**：当前实现 `MembershipDirectorySigner::hmac_sha256`，基于快照核心字段做 canonical CBOR 签名。
- **发布入口**：`publish_membership_change_with_dht_signed` 同步发布带签名广播并写入 DHT。
- **恢复策略**：`restore_membership_from_dht_verified` 支持 `trusted_requesters` 与 `require_signature` 策略。
- **来源约束**：恢复前校验 `requester_id` 必须在快照 validator 集合内；若配置白名单则必须命中。
- **兼容模式**：未启用策略时仍保留旧恢复入口，便于渐进迁移。

## 成员目录快照密钥轮换与审计（草案）
- **key_id 扩展**：成员目录广播与 DHT 快照增加可选 `signature_key_id`，用于标识签名密钥版本。
- **多密钥验签**：`MembershipDirectorySignerKeyring` 支持 active key 签名和多 key 验签，兼容轮换窗口。
- **策略控制**：恢复策略新增 `require_signature_key_id` 与 `accepted_signature_key_ids`，可限定只接受指定 key_id。
- **发布入口**：提供 keyring 版本发布接口，默认使用 active key 生成签名并写入 `signature_key_id`。
- **审计结果**：恢复流程输出 `MembershipSnapshotAuditRecord`，统一记录 `missing/applied/ignored/rejected`。
- **兼容模式**：对历史无 key_id 快照仍可验签；生产环境建议开启 key_id 强制策略。

## 成员目录审计持久化与吊销传播（草案）
- **审计持久化**：新增 `MembershipAuditStore` 抽象与 `InMemoryMembershipAuditStore` 参考实现。
- **恢复入口**：新增 `restore_membership_from_dht_verified_with_audit_store`，在恢复后自动写入审计记录。
- **吊销通道**：新增 gossipsub topic `aw.<world_id>.membership.revoke`，传播 key_id 吊销事件。
- **吊销同步**：`MembershipSyncClient` 新增发布/订阅/同步吊销消息能力，支持批量消费。
- **验签拦截**：`MembershipDirectorySignerKeyring` 增加 revoked key 集，吊销 key 不可签名且不可验签。
- **策略兜底**：恢复策略新增 `revoked_signature_key_ids`，即使未同步吊销广播也可拒绝失效 key_id。

## 成员目录吊销来源鉴权与审计落盘归档（草案）
- **授权校验**：吊销同步策略支持 requester 信任与签名策略组合校验，拒绝伪造来源。
- **落盘实现**：新增 `FileMembershipAuditStore`（JSONL），支持按 world_id 的 append/list 归档查询。
- **接口扩展**：支持 `publish_key_revocation_signed(_by_key_id/_with_keyring)` 多签发入口。
- **可维护性**：成员目录校验辅助逻辑拆分到 `distributed_membership_sync/logic.rs`，保持主文件规模可维护。

## 成员目录吊销授权治理与跨节点对账（草案）
- **授权治理**：`MembershipRevocationSyncPolicy` 新增 `authorized_requesters`，在 trusted 之外提供治理授权白名单。
- **对账通道**：新增 gossipsub topic `aw.<world_id>.membership.reconcile`，用于广播 revoked key 集 checkpoint。
- **对账结构**：`MembershipRevocationCheckpointAnnounce { node_id, revoked_key_ids, revoked_set_hash }`。
- **对账策略**：新增 `MembershipRevocationReconcilePolicy`（trusted_nodes + auto_revoke_missing_keys）。
- **对账报告**：新增 `MembershipRevocationReconcileReport`，记录 `in_sync/diverged/merged/rejected`。
- **收敛机制**：`reconcile_revocations_with_policy` 可在 divergence 时自动补齐本地缺失吊销 key，实现跨节点状态收敛。

## 成员目录吊销异常告警与对账调度自动化（草案）
- **告警策略**：新增 `MembershipRevocationAlertPolicy`（`warn_diverged_threshold`、`critical_rejected_threshold`）。
- **告警结构**：新增 `MembershipRevocationAnomalyAlert` 与 `MembershipRevocationAlertSeverity`，统一表达 warn/critical 异常。
- **告警评估**：新增 `evaluate_revocation_reconcile_alerts(...)`，将对账报告映射为结构化告警列表。
- **调度策略**：新增 `MembershipRevocationReconcileSchedulePolicy`（checkpoint/reconcile 间隔）。
- **调度状态**：新增 `MembershipRevocationReconcileScheduleState`，记录最近 checkpoint/reconcile 执行时间。
- **调度执行**：新增 `run_revocation_reconcile_schedule(...)`，自动判定到期任务并输出 `MembershipRevocationScheduledRunReport`。

## 成员目录吊销告警上报与调度状态持久化（草案）
- **告警上报**：新增 `MembershipRevocationAlertSink` 抽象，提供内存与 JSONL 文件实现。
- **状态存储**：新增 `MembershipRevocationScheduleStateStore` 抽象，提供内存与 JSON 文件实现。
- **编排入口**：新增 `run_revocation_reconcile_schedule_with_store_and_alerts(...)`，打通 load → run → save → emit。
- **上报接口**：新增 `emit_revocation_reconcile_alerts(...)`，统一批量告警写入 sink。
- **持久化维度**：文件落盘按 `world_id/node_id` 切分，便于节点级恢复与诊断。

## 成员目录吊销告警抑制去重与调度多节点协同（草案）
- **去重策略**：新增 `MembershipRevocationAlertDedupPolicy` 与 `MembershipRevocationAlertDedupState`。
- **去重入口**：新增 `deduplicate_revocation_alerts(...)`，按 `world/node/code` + 时间窗口抑制重复告警。
- **协同抽象**：新增 `MembershipRevocationScheduleCoordinator` 与 `InMemoryMembershipRevocationScheduleCoordinator`。
- **协同编排**：新增 `run_revocation_reconcile_coordinated(...)`，先抢占协调锁再执行 schedule/store/alert。
- **运行报告**：新增 `MembershipRevocationCoordinatedRunReport`，反馈是否获得执行权与实际告警发出数量。


## 成员目录吊销协同状态外部存储与告警恢复机制（草案）
- **协同状态存储**：新增 `MembershipRevocationCoordinatorStateStore`，支持内存/文件 lease 状态持久化。
- **Store 协调器**：新增 `StoreBackedMembershipRevocationScheduleCoordinator`，基于外部状态实现跨进程协同锁。
- **恢复存储**：新增 `MembershipRevocationAlertRecoveryStore`，记录待重放告警队列。
- **恢复发送**：新增 `emit_revocation_reconcile_alerts_with_recovery(...)`，先重放 pending，再发送新告警，失败回写队列。
- **协同编排**：新增 `run_revocation_reconcile_coordinated_with_recovery(...)`，打通协同调度 + 去重 + 恢复发送。
- **运行报告**：新增 `MembershipRevocationAlertRecoveryReport` 与 `MembershipRevocationCoordinatedRecoveryRunReport`。

## 成员目录吊销恢复队列容量治理与告警 ACK 重试（草案）
- **恢复队列元素**：新增 `MembershipRevocationPendingAlert`，持久化 `attempt/next_retry_at_ms/last_error` 等重试元数据。
- **ACK 重试策略**：新增 `MembershipRevocationAlertAckRetryPolicy`（`max_pending_alerts/max_retry_attempts/retry_backoff_ms`）。
- **发送入口扩展**：新增 `emit_revocation_reconcile_alerts_with_recovery_and_ack_retry(...)`，支持到期重试、失败退避、容量裁剪。
- **协同编排扩展**：新增 `run_revocation_reconcile_coordinated_with_recovery_and_ack_retry(...)`，在协调调度链路中接入 ACK 重试策略。
- **兼容策略**：文件 recovery store 支持读取旧版 `Vec<MembershipRevocationAnomalyAlert>` 格式并自动升级为 pending 结构。
- **运行报告扩展**：`MembershipRevocationAlertRecoveryReport`/`MembershipRevocationCoordinatedRecoveryRunReport` 增加 `deferred/dropped_capacity/dropped_retry_limit` 指标。
