# P2P/区块链链路安全硬化（2026-02-23，设计文档）

## 目标
- 修复 replication 写入链路中的“先推进 guard 后写入”状态污染问题，保证失败不产生半更新。
- 将 Node 共识与复制链路从“签名可选/绑定可选”收口到“生产默认严格绑定与授权”。
- 为 replication `fetch-commit`/`fetch-blob` 增加请求鉴权，降低匿名拉取与枚举风险。
- 为网络订阅队列补齐容量上限，避免高频 topic spam 导致内存无界增长。
- 收紧 membership DHT 恢复默认策略，避免未签名快照在默认路径被接受。

## 范围

### In Scope
- `crates/agent_world_distfs/src/replication.rs`
  - `apply_replication_record` 改为原子语义（失败不污染 guard）。
- `crates/agent_world_node/src/replication.rs`
  - 增加远端 writer allowlist 与 fetch 请求签名鉴权数据模型。
  - 增加远端复制消息授权校验（签名有效 + writer 授权）。
- `crates/agent_world_node/src/lib.rs`
  - 启动阶段把 validator signer 绑定与 replication writer allowlist 接线。
  - `sync_missing_replication_commits` / storage challenge 网络探测改为带鉴权请求。
  - fetch handler 增加请求鉴权校验。
- `crates/agent_world_node/src/types.rs`
  - 保持配置 API 向后兼容，强化“签名模式下 signer 绑定完整性”语义约束。
- `crates/agent_world/src/bin/world_viewer_live.rs`
  - 为 triad/single 模式生成并注入 validator signer 公钥绑定（与节点签名密钥一致）。
- `crates/agent_world_proto/src/distributed_net.rs`
  - 网络订阅 inbox 引入有界队列策略。
- `crates/agent_world_consensus/src/membership.rs`
  - `restore_membership_from_dht` 默认策略切换为 require signature。

### Out of Scope
- 不引入新密码学算法（继续使用当前 ed25519 / HMAC 体系）。
- 不改造 libp2p 底层握手协议与 peer 信任模型。
- 不重构大型文件拆分（本次聚焦安全行为改造与测试闭环）。

## 接口 / 数据

### 1) Replication Guard 原子化
- `apply_replication_record` 调整为：
  - 在局部副本上执行 `validate_and_advance`；
  - 完成 hash 校验与 `store.write_file` 后再提交 guard。
- 失败路径保证：
  - 任何 `Err` 不改变外部 `guard`。

### 2) Signed Consensus 模式下的 signer 绑定完整性
- 当节点启用共识签名强制（由 replication 签名配置驱动）时：
  - `validator_signer_public_keys` 必须覆盖全部 validator；
  - 本地节点 signer 公钥必须与自身 validator 绑定一致；
  - 不满足时启动失败。

### 3) Replication Writer 授权
- `NodeReplicationConfig` 增加远端 writer allowlist（ed25519 公钥 hex 集）。
- 远端 replication 消息校验增加：
  - 签名校验通过；
  - `record.writer_id == signature.public_key_hex`；
  - `record.writer_id` 命中 allowlist。

### 4) Fetch 请求鉴权
- `FetchCommitRequest` / `FetchBlobRequest` 增加鉴权字段（请求方公钥 + 签名）。
- 客户端发起请求时对请求体签名；服务端根据 allowlist 验证签名后处理。
- 缺失或校验失败返回 `ErrBadRequest`。

### 5) 网络订阅容量上限
- `NetworkSubscription` drain inbox 改为有界缓存（超过上限淘汰最旧消息）。
- 默认上限覆盖高频 topic 场景，避免无界 `Vec<Vec<u8>>` 累积。

### 6) Membership DHT 恢复默认策略
- `restore_membership_from_dht` 默认使用 `require_signature=true`。
- 旧行为可通过显式传入宽松策略 API 保留（受调用方控制）。

## 里程碑
- M0：T0 建档完成。
- M1：T1 完成 replication guard 原子化 + 测试。
- M2：T2 完成 signed consensus signer 绑定强制 + viewer 配置接线。
- M3：T3 完成 replication writer allowlist 授权校验。
- M4：T4 完成 fetch 请求签名鉴权链路。
- M5：T5 完成网络订阅有界队列与 membership 默认策略收紧。
- M6：T6 完成测试回归、文档状态回写与 devlog。

## 风险
- 兼容性风险：签名与绑定默认收紧后，未配置 signer 映射或混用旧节点会被拒绝，需要同步升级。
- 运维风险：triad 场景需要稳定可复现的节点密钥派生策略，避免跨进程配置不一致。
- 可用性风险：鉴权失败会提升拒绝率，需通过清晰错误信息辅助定位配置问题。
